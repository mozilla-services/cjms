use sqlx::{Pool, Postgres};

use crate::{cj::client::CJClient, telemetry::StatsD};

pub async fn verify_reports_with_cj(
    _db_pool: &Pool<Postgres>,
    _cj_client: &CJClient,
    _statsd: &StatsD,
) {
    // Get the date range with which to query cj

    // Get the list of subscriptions and the list of refunds we're looking for

    // Query CJ

    // Add the results count to statsd

    // Iterate through the subscriptions updating as we go

    // Iterate through the refunds updating as we go
}
