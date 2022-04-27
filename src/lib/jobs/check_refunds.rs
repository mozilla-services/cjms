use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    bigquery::client::{BQClient, BQError, ResultSet},
    error_and_incr, info_and_incr,
    models::{
        refunds::{PartialRefund, Refund, RefundModel},
        status_history::{Status, UpdateStatus},
        subscriptions::SubscriptionModel,
    },
    telemetry::{LogKey, StatsD},
};

fn make_refund_from_bq_row(rs: &ResultSet) -> Result<Refund, BQError> {
    let refund = Refund::new(PartialRefund {
        id: Uuid::new_v4(),
        refund_id: rs.require_string_by_name("refund_id")?,
        subscription_id: rs.require_string_by_name("subscription_id")?,
        refund_created: rs.require_offsetdatetime_by_name("created")?,
        refund_amount: rs.require_i32_by_name("amount")?,
        refund_status: rs.get_string_by_name("status")?,
        refund_reason: rs.get_string_by_name("reason")?,
        correction_file_date: None,
    });
    Ok(refund)
}

pub async fn fetch_and_process_refunds(bq: &BQClient, db_pool: &Pool<Postgres>, statsd: &StatsD) {
    let subscriptions = SubscriptionModel { db_pool };
    let refunds = RefundModel { db_pool };

    // Get all results from bigquery table that stores refunds reports
    let query = "SELECT * FROM `cjms_bigquery.refunds_v1`;";
    let mut rs = bq.get_bq_results(query).await;
    rs.report_stats(statsd, &LogKey::CheckRefunds);
    while rs.next_row() {
        // If can't deserialize e.g. required fields are not available log and move on.
        let r = match make_refund_from_bq_row(&rs) {
            Ok(r) => {
                info_and_incr!(
                    statsd,
                    LogKey::CheckRefundsDeserializeBigQuery,
                    refund_id = r.refund_id.as_str(),
                    "Successfully deserialized refund from BigQuery row"
                );
                r
            }
            Err(e) => {
                error_and_incr!(
                    statsd,
                    LogKey::CheckRefundsDeserializeBigQueryFailed,
                    error = e,
                    "Failed to make refund for BigQuery result row. Continuing ...",
                );
                continue;
            }
        };
        // Do we have the related subscription in the subscriptions table
        let have_sub = subscriptions
            .fetch_one_by_subscription_id(&r.subscription_id)
            .await
            .is_ok();
        if !have_sub {
            error_and_incr!(
                statsd,
                LogKey::CheckRefundsSubscriptionMissingFromDatabase,
                subscription_id = r.subscription_id.as_str(),
                refund_id = r.refund_id.as_str(),
            );
            continue;
        }
        // Do we already have it in the refunds table
        match refunds.fetch_one_by_refund_id(&r.refund_id).await {
            Ok(mut refund) => {
                // Only update if data is different
                if refund.subscription_id == r.subscription_id
                    && refund.refund_created.unix_timestamp() == r.refund_created.unix_timestamp()
                    && refund.refund_amount == r.refund_amount
                    && refund.refund_status == r.refund_status
                    && refund.refund_reason == r.refund_reason
                {
                    info_and_incr!(
                        statsd,
                        LogKey::CheckRefundsRefundDataUnchanged,
                        refund_id = refund.refund_id.as_str(),
                        "Data for refund is unchanged. Continuing..."
                    );
                    continue;
                }

                info_and_incr!(
                    statsd,
                    LogKey::CheckRefundsRefundDataChanged,
                    refund_id = refund.refund_id.as_str(),
                    "Data for refund is changed. Updating..."
                );
                refund.subscription_id = r.subscription_id;
                refund.refund_created = r.refund_created;
                refund.refund_amount = r.refund_amount;
                refund.refund_status = r.refund_status;
                refund.refund_reason = r.refund_reason;
                refund.update_status(Status::NotReported);
                refund.correction_file_date = None;
                match refunds.update_refund(&refund).await {
                    Ok(_) => {
                        info_and_incr!(
                            statsd,
                            LogKey::CheckRefundsRefundUpdate,
                            refund_id = refund.refund_id.as_str(),
                            "Refund updated. Continuing..."
                        );
                    }
                    Err(e) => {
                        error_and_incr!(
                            statsd,
                            LogKey::CheckRefundsRefundUpdateFailed,
                            error = e,
                            refund_id = r.refund_id.as_str(),
                            "Error updating refund. Continuing..."
                        );
                    }
                };
            }
            Err(e) => {
                match e {
                    sqlx::Error::RowNotFound => {
                        match refunds.create_from_refund(&r).await {
                            Ok(r) => {
                                info_and_incr!(
                                    statsd,
                                    LogKey::CheckRefundsRefundCreate,
                                    refund_id = r.refund_id.as_str(),
                                    "Successfully created refund"
                                );
                            }
                            Err(e) => match e {
                                sqlx::Error::Database(e) => {
                                    // 23505 is the code for unique constraints e.g. duplicate flow id issues
                                    if e.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                                        error_and_incr!(
                                            statsd,
                                            LogKey::CheckRefundsRefundCreateDuplicateKeyViolation,
                                            error = e,
                                            refund_id = &r.refund_id.as_str(),
                                            "Duplicate key violation"
                                        );
                                    } else {
                                        error_and_incr!(
                                            statsd,
                                            LogKey::CheckRefundsRefundCreateDatabaseError,
                                            error = e,
                                            refund_id = &r.refund_id.as_str(),
                                            "Database error while creating refund. Continuing..."
                                        );
                                    }
                                    continue;
                                }
                                _ => {
                                    error_and_incr!(
                                        statsd,
                                        LogKey::CheckRefundsRefundCreateFailed,
                                        error = e,
                                        refund_id = &r.refund_id.as_str(),
                                        "Unexpected error while creating refund. Continuing..."
                                    );
                                    continue;
                                }
                            },
                        };
                    }
                    _ => {
                        error_and_incr!(
                            statsd,
                            LogKey::CheckRefundsRefundFetchFailed,
                            error = e,
                            refund_id = r.refund_id.as_str(),
                            "Error while trying to retrieve refund. Continuing..."
                        );
                    }
                }
            }
        };
    }
}
