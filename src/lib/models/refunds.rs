use serde_json::Value as JsonValue;
use sqlx::{query_as, Error, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug)]
pub struct Refund {
    id: Uuid,
    refund_id: String,
    subscription_id: String,
    refund_created: OffsetDateTime,
    refund_amount: i32,
    refund_status: Option<String>,
    refund_reason: Option<String>,
    // Note we use string and json to save in database for simplicity
    status: Option<String>,
    status_t: Option<OffsetDateTime>,
    status_history: Option<JsonValue>,
}

pub struct RefundModel<'a> {
    pub db_pool: &'a PgPool,
}

impl RefundModel<'_> {
    pub async fn create_from_refund(&self, refund: &Refund) -> Result<Refund, Error> {
        query_as!(
            Refund,
            "INSERT INTO refunds (id, refund_id, subscription_id, refund_created, refund_amount, refund_status, refund_reason, status, status_t, status_history)
			VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
			RETURNING *",
            refund.id,
            refund.refund_id,
            refund.subscription_id,
            refund.refund_created,
            refund.refund_amount,
            refund.refund_status,
            refund.refund_reason,
            refund.status,
            refund.status_t,
            refund.status_history,

        )
        .fetch_one(self.db_pool)
        .await
    }
}
