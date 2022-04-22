use sqlx::{Pool, Postgres};

use crate::{
    cj::client::CJClient,
    error_and_incr, info_and_incr,
    models::{status_history::Status, subscriptions::SubscriptionModel},
    telemetry::{LogKey, StatsD},
};

pub async fn report_subscriptions_to_cj(
    db_pool: &Pool<Postgres>,
    cj_client: &CJClient,
    statsd: &StatsD,
) {
    let subscriptions = SubscriptionModel { db_pool };
    // Intentional panic. Cannot continue if we can't retrieve subs.
    let not_reported_subscriptions = subscriptions
        .fetch_all_not_reported()
        .await
        .expect("Could not retrieve subscriptions from DB.");
    statsd.gauge(
        &LogKey::ReportSubscriptionsNNotReported,
        not_reported_subscriptions.len(),
    );

    for sub in not_reported_subscriptions {
        let next_status = match sub.aic_expires {
            Some(aic_expires) => {
                if aic_expires < sub.subscription_created {
                    info_and_incr!(
                        statsd,
                        LogKey::ReportSubscriptionsAicExpiredBeforeSubscriptionCreated,
                        sub_id = &sub.id.to_string().as_str(),
                        "AIC expired before subscription created. Will not report."
                    );
                    Status::WillNotReport
                } else {
                    Status::Reported
                }
            }
            None => {
                error_and_incr!(
                    statsd,
                    LogKey::ReportSubscriptionsSubscriptionHasNoAicExpiry,
                    sub_id = &sub.id.to_string().as_str(),
                    "Subscription does not have an AIC expiry. Will not report."
                );
                Status::WillNotReport
            }
        };
        if next_status == Status::WillNotReport {
            match subscriptions
                .update_sub_status(&sub.id, Status::WillNotReport)
                .await
            {
                Ok(_) => {
                    info_and_incr!(
                        statsd,
                        LogKey::ReportSubscriptionMarkWillNotReport,
                        sub_id = &sub.id.to_string().as_str(),
                        "Successfully marked as WillNotReport"
                    );
                }
                Err(e) => {
                    error_and_incr!(
                        statsd,
                        LogKey::ReportSubscriptionMarkWillNotReportFailed,
                        error = e,
                        sub_id = &sub.id.to_string().as_str(),
                        "Could not mark subscription as WillNotReport."
                    );
                }
            };
            continue;
        }

        let mark_not_reported = match cj_client.report_subscription(&sub).await {
            Ok(r) => {
                if r.status() == 200 {
                    match subscriptions
                        .update_sub_status(&sub.id, Status::Reported)
                        .await
                    {
                        Ok(_) => {
                            info_and_incr!(
                                statsd,
                                LogKey::ReportSubscriptionReportToCj,
                                sub_id = &sub.id.to_string().as_str(),
                                "Successfully reported sub to CJ; received 200 status"
                            );
                        }
                        Err(e) => {
                            error_and_incr!(
                                statsd,
                                LogKey::ReportSubscriptionReportToCjButCouldNotMarkReported,
                                error = e,
                                sub_id = &sub.id.to_string().as_str(),
                                "Successfully reported sub to CJ; received 200 status, but could not mark the sub as reported locally."
                            );
                        }
                    };
                    false
                } else {
                    error_and_incr!(
                        statsd,
                        LogKey::ReportSubscriptionReportToCjFailed,
                        sub_id = &sub.id.to_string().as_str(),
                        "Could not report sub to CJ; received non-200 status."
                    );
                    true
                }
            }
            Err(e) => {
                error_and_incr!(
                    statsd,
                    LogKey::ReportSubscriptionReportToCjFailed,
                    error = e,
                    sub_id = &sub.id.to_string().as_str(),
                    "Could not report sub to CJ; unknown application failure."
                );
                true
            }
        };
        if mark_not_reported {
            match subscriptions
                .update_sub_status(&sub.id, Status::NotReported)
                .await
            {
                Ok(_) => {
                    info_and_incr!(
                        statsd,
                        LogKey::ReportSubscriptionMarkNotReported,
                        sub_id = &sub.id.to_string().as_str(),
                        "Successfully marked as NotReported."
                    );
                }
                Err(e) => {
                    error_and_incr!(
                        statsd,
                        LogKey::ReportSubscriptionMarkNotReportedFailed,
                        error = e,
                        sub_id = &sub.id.to_string().as_str(),
                        "Could not mark subscription as NotReported."
                    );
                }
            }
        }
    }
}
