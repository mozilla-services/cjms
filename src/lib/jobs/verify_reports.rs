use sqlx::{Pool, Postgres};
use time::OffsetDateTime;

use crate::{
    cj::client::{convert_plan_amount_to_decimal, CJClient, CommissionDetailRecord},
    error_and_incr, info_and_incr,
    models::{
        status_history::{Status, UpdateStatus},
        subscriptions::SubscriptionModel,
    },
    telemetry::{LogKey, StatsD},
};

pub async fn verify_reports_with_cj(
    db_pool: &Pool<Postgres>,
    cj_client: &CJClient,
    statsd: &StatsD,
) {
    let subscriptions = SubscriptionModel { db_pool };

    // Get the date range with which to query cj
    let subscription_date_range = subscriptions
        .get_reported_date_range()
        .await
        .expect("Could not retrieve date range.");
    let min = subscription_date_range
        .min
        .expect("No minimum date was returned.");
    let max = subscription_date_range
        .max
        .expect("No minimum date was returned.");

    // Get the list of subscriptions and the list of refunds we're looking for
    let reported_subscriptions = subscriptions
        .fetch_all_by_status(Status::Reported)
        .await
        .expect("Could not retrieve subscriptions from DB.");

    // Query CJ
    let cj_query_result = cj_client
        .query_comission_detail_api_between_dates(min, max)
        .await;
    statsd.gauge(&LogKey::VerifyReportsCount, cj_query_result.count);

    // Iterate through the subscriptions updating as we go
    for sub in reported_subscriptions {
        let sub_id = sub.id.to_string();
        // A subscription record (as opposed to a refund) has "original: true"
        // We pull out the matching order id and original: true
        let sub_record: Vec<CommissionDetailRecord> = cj_query_result
            .records
            .iter()
            .cloned()
            .filter(|r| (r.order_id == sub_id) && r.original)
            .collect();
        let next_status = match sub_record.len() {
            0 => {
                let time_since_subscription_reported =
                    // It's ok to use unwrap, because the select does not return null status_t
                    OffsetDateTime::now_utc() - sub.get_status_t().unwrap();
                match time_since_subscription_reported.whole_hours() > 36 {
                    true => {
                        error_and_incr!(
                            statsd,
                            LogKey::VerifyReportsSubscriptionNotFound,
                            subscription_id = sub_id.as_str(),
                            "No susbscription match found 36 hours after report."
                        );
                        Status::CJNotReceived
                    }
                    false => {
                        info_and_incr!(
                            statsd,
                            LogKey::VerifyReportsSubscriptionNotFound,
                            subscription_id = sub_id.as_str(),
                            "No susbscription match found. Continue trying for up to 36 hours."
                        );
                        continue;
                    }
                }
            }
            1 => {
                info_and_incr!(
                    statsd,
                    LogKey::VerifyReportsSubscriptionFound,
                    subscription_id = sub_id.as_str(),
                    "Subscription match found."
                );
                // Verify the details are correct.
                let record = &sub_record[0];
                let correct = (record.items[0].sku == sub.plan_id)
                    && (record.sale_amount_pub_currency
                        == convert_plan_amount_to_decimal(sub.plan_amount));
                match correct {
                    true => {
                        info_and_incr!(
                            statsd,
                            LogKey::VerifyReportsSubscriptionMatched,
                            subscription_id = sub_id.as_str(),
                            "Subscription found and matched."
                        );
                        Status::CJReceived
                    }
                    false => {
                        error_and_incr!(
                            statsd,
                            LogKey::VerifyReportsSubscriptionNotMatched,
                            subscription_id = sub_id.as_str(),
                            "Subscription found but not matched."
                        );
                        Status::CJNotReceived
                    }
                }
            }
            _ => {
                error_and_incr!(
                    statsd,
                    LogKey::VerifyReportsTooManyRecords,
                    subscription_id = sub_id.as_str(),
                    "Too many records were found for the subscription."
                );
                continue;
            }
        };
        match subscriptions
            .update_sub_status(&sub.id, next_status.clone())
            .await
        {
            Ok(_) => {
                info_and_incr!(
                    statsd,
                    LogKey::VerifyReportsSubscriptionUpdated,
                    sub_id = &sub.id.to_string().as_str(),
                    status = &next_status.to_string().as_str(),
                    "Successfully updated subscription with new status."
                );
            }
            Err(e) => {
                error_and_incr!(
                    statsd,
                    LogKey::VerifyReportsSubscriptionUpdateFailed,
                    error = e,
                    status = &next_status.to_string().as_str(),
                    sub_id = &sub.id.to_string().as_str(),
                    "Subscription update with new status failed."
                );
            }
        };
    }

    // Iterate through the refunds updating as we go
}
