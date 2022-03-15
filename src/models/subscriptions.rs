use sqlx::{query_as, Error, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq)]
pub struct Subscription {
    pub aic_id: Uuid,
    pub cj_event_value: String,
    pub flow_id: String,
    pub report_timestamp: OffsetDateTime,
    pub subscription_created: OffsetDateTime,
    pub subscription_id: String,
    pub fxa_uid: String,
    pub quantity: i32,
    pub plan_id: String,
    pub plan_currency: String,
    pub plan_amount: i32,
    pub country: String,
}

pub struct SubscriptionModel<'a> {
    pub db_pool: &'a PgPool,
}

impl SubscriptionModel<'_> {
    pub async fn create_from_sub(&self, sub: &Subscription) -> Result<Subscription, Error> {
        query_as!(
            Subscription,
            r#"INSERT INTO subscriptions (aic_id, cj_event_value, flow_id, report_timestamp, subscription_created, subscription_id, fxa_uid, quantity, plan_id, plan_currency, plan_amount, country)
			VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
			RETURNING *"#,
            sub.aic_id,
            sub.cj_event_value,
            sub.flow_id,
            sub.report_timestamp,
            sub.subscription_created,
            sub.subscription_id,
            sub.fxa_uid,
            sub.quantity,
            sub.plan_id,
            sub.plan_currency,
            sub.plan_amount,
            sub.country
        )
        .fetch_one(self.db_pool)
        .await
    }

    pub async fn fetch_one_by_aic_id(&self, aic_id: &Uuid) -> Result<Subscription, Error> {
        query_as!(
            Subscription,
            "SELECT * FROM subscriptions WHERE aic_id = $1",
            aic_id
        )
        .fetch_one(self.db_pool)
        .await
    }
}
