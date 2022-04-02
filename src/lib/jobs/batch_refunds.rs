use sqlx::{Pool, Postgres};

use crate::models::refunds::RefundModel;

pub async fn batch_refunds_by_day(db_pool: &Pool<Postgres>) {
    let _refunds = RefundModel { db_pool };
}
