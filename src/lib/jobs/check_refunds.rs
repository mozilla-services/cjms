use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    bigquery::client::{BQClient, BQError, ResultSet},
    models::{
        refunds::{PartialRefund, Refund, RefundModel},
        subscriptions::SubscriptionModel, status_history::{UpdateStatus, Status},
    },
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

pub async fn fetch_and_process_refunds(bq: BQClient, db_pool: &Pool<Postgres>) {
    let subscriptions = SubscriptionModel { db_pool };
    let refunds = RefundModel { db_pool };

    // Get all results from bigquery table that stores refunds reports
    let query = "SELECT * FROM `cjms_bigquery.refunds_v1`;";
    let mut rs = bq.get_bq_results(query).await;
    while rs.next_row() {
        // If can't deserialize e.g. required fields are not available log and move on.
        let r = match make_refund_from_bq_row(&rs) {
            Ok(r) => {
                // TODO - LOGGING
                println!(
                    "Successfully deserialized refund from bq row: {}",
                    r.refund_id
                );
                r
            }
            Err(e) => {
                // TODO - LOGGING - Log information and get a metric
                println!(
                    "Failed to make refund for bq result row. {:?}. Continuing...",
                    e
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
            // TODO - LOGGING
            println!(
                "Subscription {} is not in subscriptions table. Refund {}. Continuing....",
                r.subscription_id, r.refund_id
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
                    println!(
                        "Data for refund {} is unchanged. Continuing....",
                        refund.refund_id
                    );
                    continue;
                }

                println!(
                    "Data for refund {} is changed. Updating....",
                    refund.refund_id
                );
                refund.subscription_id = r.subscription_id;
                refund.refund_created = r.refund_created;
                refund.refund_amount = r.refund_amount;
                refund.refund_status = r.refund_status;
                refund.refund_reason = r.refund_reason;
                refund.update_status(Status::NotReported);
                refund.correction_file_date = None;
                match refunds.update_refund(&mut refund).await {
                    Ok(_) => {
                        println!("Refund {} updated. Continuing...", refund.refund_id);
                    }
                    Err(e) => {
                        // TODO - LOGGING
                        println!(
                            "Error updating refund: {}. {}. Continuing....",
                            r.refund_id, e
                        );
                    }
                };
            }
            Err(e) => {
                match e {
                    sqlx::Error::RowNotFound => {
                        match refunds.create_from_refund(&r).await {
                            Ok(r) => {
                                // TODO - LOGGING
                                println!("Successfully created refund: {}.", r.refund_id);
                            }
                            Err(e) => match e {
                                sqlx::Error::Database(e) => {
                                    // 23505 is the code for unique constraints e.g. duplicate flow id issues
                                    if e.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                                        // TODO - LOGGING - add some specific logging / metrics around duplicate key issues.
                                        // This could help us see that we have an ETL issue.
                                        println!("Duplicate Key Violation");
                                    }
                                    println!(
                                "DatabaseError error while creating refund {:?}. Continuing...",
                                e
                            );
                                    continue;
                                }
                                _ => {
                                    println!(
                                "Unexpected error while creating refund {:?}. Continuing...",
                                e
                            );
                                    continue;
                                }
                            },
                        };
                    }
                    _ => {
                        // TODO - LOGGING
                        println!(
                            "Error when trying to retrieve: {}. {}. Continuing....",
                            r.refund_id, e
                        );
                    }
                }
            }
        };
    }
}
