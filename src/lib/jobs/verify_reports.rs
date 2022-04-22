use sqlx::{Pool, Postgres};

use crate::telemetry::StatsD;

pub async fn verify_reports_with_cj(_db_pool: &Pool<Postgres>, _statsd: &StatsD) {}
