use serde_json::Value as JsonValue;
use sqlx::{query_as, Error, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::status_history::{Status, UpdateStatus};

#[derive(Debug)]
pub struct Subscription {
    pub id: Uuid,
    pub flow_id: String,
    pub subscription_id: String,
    pub report_timestamp: OffsetDateTime,
    pub subscription_created: OffsetDateTime,
    // Note this is a hash
    pub fxa_uid: String,
    pub quantity: i32,
    pub plan_id: String,
    pub plan_currency: String,
    pub plan_amount: i32,
    pub country: Option<String>,
    pub aic_id: Option<Uuid>,
    pub aic_expires: Option<OffsetDateTime>,
    pub cj_event_value: Option<String>,
    // Note we use string and json to save in database for simplicity
    status: Option<String>,
    status_history: Option<JsonValue>,
}
impl PartialEq for Subscription {
    fn eq(&self, other: &Self) -> bool {
        let simple_match = self.id == other.id &&
        self.flow_id == other.flow_id &&
        self.subscription_id == other.subscription_id &&
        // When timestamps go in and out of database they lose precision to milliseconds
        self.report_timestamp.unix_timestamp() == other.report_timestamp.unix_timestamp() &&
        self.subscription_created.unix_timestamp() == other.subscription_created.unix_timestamp() &&
        self.fxa_uid == other.fxa_uid &&
        self.quantity == other.quantity &&
        self.plan_id == other.plan_id &&
        self.plan_currency == other.plan_currency &&
        self.plan_amount == other.plan_amount &&
        self.country == other.country &&
        self.aic_id == other.aic_id &&
        self.cj_event_value == other.cj_event_value &&
        self.status == other.status
        // Compare manually if needed
        // self.status_history == other.status_history
        ;
        let aic_expires_match = match self.aic_expires {
            Some(self_v) => match other.aic_expires {
                Some(other_v) => self_v.unix_timestamp() == other_v.unix_timestamp(),
                None => false,
            },
            None => other.aic_expires.is_none(),
        };
        aic_expires_match && simple_match
    }
}
impl Eq for Subscription {}

impl UpdateStatus for Subscription {
    fn get_raw_status(&self) -> Option<String> {
        self.status.clone()
    }

    fn get_raw_status_history(&self) -> Option<JsonValue> {
        self.status_history.clone()
    }

    fn set_raw_status(&mut self, v: Option<String>) {
        self.status = v;
    }

    fn set_raw_status_history(&mut self, v: Option<JsonValue>) {
        self.status_history = v;
    }
}

impl Subscription {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        flow_id: String,
        subscription_id: String,
        report_timestamp: OffsetDateTime,
        subscription_created: OffsetDateTime,
        fxa_uid: String,
        quantity: i32,
        plan_id: String,
        plan_currency: String,
        plan_amount: i32,
        country: Option<String>,
        aic_id: Option<Uuid>,
        aic_expires: Option<OffsetDateTime>,
        cj_event_value: Option<String>,
    ) -> Self {
        let mut sub = Subscription {
            id,
            flow_id,
            subscription_id,
            report_timestamp,
            subscription_created,
            fxa_uid,
            quantity,
            plan_id,
            plan_currency,
            plan_amount,
            country,
            aic_id,
            aic_expires,
            cj_event_value,
            status: None,
            status_history: None,
        };
        sub.update_status(Status::NotReported);
        sub
    }
}

pub struct SubscriptionModel<'a> {
    pub db_pool: &'a PgPool,
}

impl SubscriptionModel<'_> {
    pub async fn create_from_sub(&self, sub: &Subscription) -> Result<Subscription, Error> {
        query_as!(
            Subscription,
            "INSERT INTO subscriptions (
                id,
                flow_id,
                subscription_id,
                report_timestamp,
                subscription_created,
                fxa_uid,
                quantity,
                plan_id,
                plan_currency,
                plan_amount,
                country,
                aic_id,
                aic_expires,
                cj_event_value,
                status,
                status_history
             )
			VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
			RETURNING *",
            sub.id,
            sub.flow_id,
            sub.subscription_id,
            sub.report_timestamp,
            sub.subscription_created,
            sub.fxa_uid,
            sub.quantity,
            sub.plan_id,
            sub.plan_currency,
            sub.plan_amount,
            sub.country,
            sub.aic_id,
            sub.aic_expires,
            sub.cj_event_value,
            sub.status,
            sub.status_history,
        )
        .fetch_one(self.db_pool)
        .await
    }

    pub async fn fetch_one_by_id(&self, id: &Uuid) -> Result<Subscription, Error> {
        query_as!(
            Subscription,
            "SELECT * FROM subscriptions WHERE id = $1",
            id
        )
        .fetch_one(self.db_pool)
        .await
    }

    pub async fn fetch_one_by_flow_id(&self, flow_id: &str) -> Result<Subscription, Error> {
        query_as!(
            Subscription,
            "SELECT * FROM subscriptions WHERE flow_id = $1",
            flow_id
        )
        .fetch_one(self.db_pool)
        .await
    }

    pub async fn fetch_all(&self) -> Result<Vec<Subscription>, Error> {
        query_as!(Subscription, "SELECT * FROM subscriptions")
            .fetch_all(self.db_pool)
            .await
    }

    pub async fn fetch_all_not_reported(&self) -> Result<Vec<Subscription>, Error> {
        query_as!(
            Subscription,
            "SELECT * FROM subscriptions WHERE status = 'NotReported'"
        )
        .fetch_all(self.db_pool)
        .await
    }

    pub async fn update_sub_status(
        &self,
        id: &Uuid,
        new_status: Status,
    ) -> Result<Subscription, Error> {
        let mut sub = self.fetch_one_by_id(id).await?;
        sub.update_status(new_status);
        query_as!(
            Subscription,
            r#"UPDATE subscriptions
            SET
                status = $1,
                status_history = $2
            WHERE id = $3
			RETURNING *"#,
            sub.status,
            sub.status_history,
            id,
        )
        .fetch_one(self.db_pool)
        .await
    }
}
