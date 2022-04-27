use sqlx::{Pool, Postgres};

use uuid::Uuid;

use crate::{
    bigquery::client::{BQClient, BQError, ResultSet},
    error_and_incr, info_and_incr,
    models::{
        aic::AICModel,
        subscriptions::{PartialSubscription, Subscription, SubscriptionModel},
    },
    telemetry::{LogKey, StatsD},
};

fn make_subscription_from_bq_row(rs: &ResultSet) -> Result<Subscription, BQError> {
    let sub = Subscription::new(PartialSubscription {
        id: Uuid::new_v4(),
        flow_id: rs.require_string_by_name("flow_id")?,
        subscription_id: rs.require_string_by_name("subscription_id")?,
        report_timestamp: rs.require_offsetdatetime_by_name("report_timestamp")?,
        subscription_created: rs.require_offsetdatetime_by_name("subscription_created")?,
        fxa_uid: rs.require_string_by_name("fxa_uid")?,
        quantity: rs.require_i32_by_name("quantity")?,
        plan_id: rs.require_string_by_name("plan_id")?,
        plan_currency: rs.require_string_by_name("plan_currency")?,
        plan_amount: rs.require_i32_by_name("plan_amount")?,
        country: rs.get_string_by_name("country")?,
        aic_id: None,
        aic_expires: None,
        cj_event_value: None,
    });
    Ok(sub)
}

pub async fn fetch_and_process_new_subscriptions(
    bq: &BQClient,
    db_pool: &Pool<Postgres>,
    statsd: &StatsD,
) {
    let subscriptions = SubscriptionModel { db_pool };
    let aics = AICModel { db_pool };
    // Get all results from bigquery table that stores new subscription reports
    let query = "SELECT * FROM `cjms_bigquery.subscriptions_v1`;";
    let mut rs = bq.get_bq_results(query).await;
    rs.report_stats(statsd, &LogKey::CheckSubscriptions);
    while rs.next_row() {
        // If can't deserialize e.g. required fields are not available log and move on.
        let mut sub = match make_subscription_from_bq_row(&rs) {
            Ok(sub) => {
                info_and_incr!(
                    statsd,
                    LogKey::CheckSubscriptionsDeserializeBigQuery,
                    subscription_id = sub.id.to_string().as_str(),
                    "Successfully deserialized subscription from BigQuery row",
                );
                sub
            }
            Err(e) => {
                error_and_incr!(
                    statsd,
                    LogKey::CheckSubscriptionsDeserializeBigQueryFailed,
                    error = e,
                    "Failed to make subscription for BigQuery result row. Continuing...",
                );
                continue;
            }
        };
        let (aic, aic_found_in_archive) = match aics.fetch_one_by_flow_id(&sub.flow_id).await {
            Ok(aic) => {
                info_and_incr!(
                    statsd,
                    LogKey::CheckSubscriptionsAicFetch,
                    aic_id = aic.id.to_string().as_str(),
                    "Successfully fetched aic",
                );
                (aic, false)
            }
            Err(_) => match aics.fetch_one_by_flow_id_from_archive(&sub.flow_id).await {
                Ok(aic) => {
                    info_and_incr!(
                        statsd,
                        LogKey::CheckSubscriptionsAicFetchFromArchive,
                        aic_id = aic.id.to_string().as_str(),
                        "AIC was fetched from archive table.",
                    );
                    (aic, true)
                }
                Err(e) => {
                    error_and_incr!(
                        statsd,
                        LogKey::CheckSubscriptionsAicFetchFailed,
                        error = e,
                        "Error getting aic for subscription. Continuing...",
                    );
                    continue;
                }
            },
        };
        sub.aic_id = Some(aic.id);
        sub.cj_event_value = Some(aic.cj_event_value.clone());
        sub.aic_expires = Some(aic.expires);

        // Archive the AIC
        if !aic_found_in_archive {
            match aics.archive_aic(&aic).await {
                Ok(_) => {
                    info_and_incr!(
                        statsd,
                        LogKey::CheckSubscriptionsAicArchive,
                        aic_id = aic.id.to_string().as_str(),
                        "Successfully archived aic",
                    );
                }
                Err(e) => {
                    error_and_incr!(
                        statsd,
                        LogKey::CheckSubscriptionsAicArchiveFailed,
                        error = e,
                        aic_id = aic.id.to_string().as_str(),
                        "Failed to archive aic entry. Continuing...",
                    );
                    continue;
                }
            };
        }
        // Save the new subscription entry
        match subscriptions.create_from_sub(&sub).await {
            Ok(sub) => {
                info_and_incr!(
                    statsd,
                    LogKey::CheckSubscriptionsSubscriptionCreate,
                    sub_id = sub.id.to_string().as_str(),
                    "Successfully created subscription"
                );
            }
            Err(e) => match e {
                sqlx::Error::Database(e) => {
                    // 23505 is the code for unique constraints e.g. duplicate flow id issues
                    if e.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                        error_and_incr!(
                            statsd,
                            LogKey::CheckSubscriptionsSubscriptionCreateDuplicateKeyViolation,
                            error = e,
                            "Duplicate key violation"
                        );
                    }
                    error_and_incr!(
                        statsd,
                        LogKey::CheckSubscriptionsSubscriptionCreateDatabaseError,
                        error = e,
                        "Database error while creating subscription. Continuing..."
                    );
                    continue;
                }
                _ => {
                    error_and_incr!(
                        statsd,
                        LogKey::CheckSubscriptionsSubscriptionCreateFailed,
                        error = e,
                        "Unexpected error while creating subscription. Continuing...",
                    );
                    continue;
                }
            },
        };
    }
}
