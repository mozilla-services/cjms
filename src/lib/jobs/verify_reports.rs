use sqlx::{Pool, Postgres};
use time::OffsetDateTime;

use crate::{
    cj::client::{convert_amount_to_decimal, CJClient, CommissionDetailRecord},
    error_and_incr, info_and_incr,
    models::{
        refunds::RefundModel,
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
    let refunds = RefundModel { db_pool };

    // Get the list of subscriptions and the list of refunds we're looking for
    let reported_subscriptions = subscriptions
        .fetch_all_by_status(Status::Reported)
        .await
        .expect("Could not retrieve subscriptions from DB.");
    let reported_refunds = refunds
        .fetch_all_by_status(Status::Reported)
        .await
        .expect("Could not retrieve refunds from DB.");

    // Get the date range with which to query cj
    let mut min_sub = None;
    let mut max_sub = None;
    let mut min_refund = None;
    let mut max_refund = None;

    if !reported_subscriptions.is_empty() {
        let subscription_date_range = subscriptions
            .get_reported_date_range()
            .await
            .expect("Could not retrieve date range.");
        min_sub = Some(
            subscription_date_range
                .min
                .expect("No minimum date was returned."),
        );
        max_sub = Some(
            subscription_date_range
                .max
                .expect("No maximum date was returned."),
        );
    }
    if !reported_refunds.is_empty() {
        let refund_date_range = refunds
            .get_reported_date_range()
            .await
            .expect("Could not retrieve date range.");
        min_refund = Some(
            refund_date_range
                .min
                .expect("No minimum date was returned."),
        );
        max_refund = Some(
            refund_date_range
                .max
                .expect("No maximum date was returned."),
        );
    }
    let mins: Vec<OffsetDateTime> = [min_sub, min_refund].iter().cloned().flatten().collect();
    let maxs: Vec<OffsetDateTime> = [max_sub, max_refund].iter().cloned().flatten().collect();
    let min = match mins.iter().cloned().min() {
        Some(t) => t,
        None => {
            info_and_incr!(
                statsd,
                LogKey::VerifyReportsNoCount,
                n_subscriptions = reported_subscriptions.len(),
                n_refunds = reported_refunds.len(),
                "No maximum date. So nothing to check. Aborting..."
            );
            return;
        }
    };
    let max = match maxs.iter().cloned().max() {
        Some(t) => t,
        None => {
            info_and_incr!(
                statsd,
                LogKey::VerifyReportsNoCount,
                n_subscriptions = reported_subscriptions.len(),
                n_refunds = reported_refunds.len(),
                "No maximum date. So nothing to check. Aborting..."
            );
            return;
        }
    };

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
                            "No susbscription match found. Continue trying for up to 36 hours. Continuing..."
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
                        == convert_amount_to_decimal(sub.plan_amount));
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
                    "Too many records were found for the subscription. Continuing..."
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
    for refund in reported_refunds {
        let related_sub = match subscriptions
            .fetch_one_by_subscription_id(&refund.subscription_id)
            .await
        {
            Ok(sub) => sub,
            Err(e) => {
                error_and_incr!(
                    statsd,
                    LogKey::VerifyRefundsSubscriptionMissingFromDatabase,
                    error = e,
                    stripe_subscription_id = refund.subscription_id.as_str(),
                    "Could not find subscription that pairs with refund. Continuing..."
                );
                continue;
            }
        };
        let related_sub_id = related_sub.id.to_string();
        // A refund record (as opposed to a subscription) has "original: false"
        // We pull out the matching order id and original: false
        let refund_record: Vec<CommissionDetailRecord> = cj_query_result
            .records
            .iter()
            .cloned()
            .filter(|r| (r.order_id == related_sub_id) && !r.original)
            .collect();
        let next_status = match refund_record.len() {
            0 => {
                let time_since_refund_reported =
                    // It's ok to use unwrap, because the select does not return null status_t
                    OffsetDateTime::now_utc() - refund.get_status_t().unwrap();
                match time_since_refund_reported.whole_hours() > 36 {
                    true => {
                        error_and_incr!(
                            statsd,
                            LogKey::VerifyReportsRefundNotFound,
                            refund_id = refund.id.to_string().as_str(),
                            "No refund match found 36 hours after report."
                        );
                        Status::CJNotReceived
                    }
                    false => {
                        info_and_incr!(
                            statsd,
                            LogKey::VerifyReportsRefundNotFound,
                            refund_id = refund.id.to_string().as_str(),
                            "No refund match found. Continue trying for up to 36 hours. Continuing..."
                        );
                        continue;
                    }
                }
            }
            1 => {
                info_and_incr!(
                    statsd,
                    LogKey::VerifyReportsRefundFound,
                    refund_id = refund.id.to_string().as_str(),
                    "Refund match found."
                );
                // Verify the details are correct.
                let record = &refund_record[0];
                let correct = (record.correction_reason
                    == Some(String::from("RETURNED_MERCHANDISE")))
                    && (record.items[0].sku == related_sub.plan_id)
                    && (record.sale_amount_pub_currency
                        == convert_amount_to_decimal(-refund.refund_amount));
                match correct {
                    true => {
                        info_and_incr!(
                            statsd,
                            LogKey::VerifyReportsRefundMatched,
                            refund_id = refund.id.to_string().as_str(),
                            "Refund found and matched."
                        );
                        Status::CJReceived
                    }
                    false => {
                        error_and_incr!(
                            statsd,
                            LogKey::VerifyReportsRefundNotMatched,
                            refund_id = refund.id.to_string().as_str(),
                            "Refund found but not matched."
                        );
                        Status::CJNotReceived
                    }
                }
            }
            _ => {
                error_and_incr!(
                    statsd,
                    LogKey::VerifyReportsTooManyRecords,
                    refund_id = refund.id.to_string().as_str(),
                    "Too many records were found for the refund. Continuing..."
                );
                continue;
            }
        };
        match refunds
            .update_refund_status(&refund.refund_id, next_status.clone())
            .await
        {
            Ok(_) => {
                info_and_incr!(
                    statsd,
                    LogKey::VerifyReportsRefundUpdated,
                    refund_id = &refund.id.to_string().as_str(),
                    status = &next_status.to_string().as_str(),
                    "Successfully updated refund with new status."
                );
            }
            Err(e) => {
                error_and_incr!(
                    statsd,
                    LogKey::VerifyReportsRefundUpdateFailed,
                    error = e,
                    status = &next_status.to_string().as_str(),
                    refund_id = &refund.id.to_string().as_str(),
                    "Refund update with new status failed."
                );
            }
        };
    }
}
