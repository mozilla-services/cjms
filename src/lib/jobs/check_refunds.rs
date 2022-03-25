use sqlx::{Pool, Postgres};

use crate::bigquery::client::BQClient;

// Throw an error if required fields are not available
pub async fn fetch_and_process_refunds(_bq: BQClient, _db_pool: &Pool<Postgres>) {}
