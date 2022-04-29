use sqlx::{Pool, Postgres};

use crate::{
    cj::client::CJClient,
    models::{status_history::Status, subscriptions::SubscriptionModel},
    telemetry::StatsD,
};

pub async fn verify_reports_with_cj(
    db_pool: &Pool<Postgres>,
    cj_client: &CJClient,
    _statsd: &StatsD,
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
    let _reported_subscriptions = subscriptions
        .fetch_all_by_status(Status::Reported)
        .await
        .expect("Could not retrieve subscriptions from DB.");

    // Query CJ
    let _cj_query_result = cj_client
        .query_comission_detail_api_between_dates(min, max)
        .await;

    // Get from result to processed data

    // Add the results count to statsd

    // Iterate through the subscriptions updating as we go

    // Iterate through the refunds updating as we go
}
