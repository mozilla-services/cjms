use sqlx::{query_as, Error, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::actions::bigquery::model::ResultSet;

#[derive(Debug)]
pub struct Subscription {
    pub id: Uuid,
    pub report_timestamp: OffsetDateTime,
    pub subscription_start_date: OffsetDateTime,
    pub fxa_uid: String,
    pub quantity: i32,
    pub plan_id: String,
    pub plan_currency: String,
    pub plan_amount: i32,
    pub country: String,
    pub promotion_codes: String,
}

pub struct SubscriptionModel<'a> {
    pub db_pool: &'a PgPool,
}

impl SubscriptionModel<'_> {
    pub async fn create_from_big_query_resultset(
        &self,
        rs: &ResultSet,
    ) -> Result<Subscription, Error> {
        let id = Uuid::new_v4();
        let report_timestamp = OffsetDateTime::from_unix_timestamp(
            rs.get_i64_by_name("report_timestamp").unwrap().unwrap(),
        );
        let subscription_start_date = OffsetDateTime::from_unix_timestamp(
            rs.get_i64_by_name("subscription_start_date")
                .unwrap()
                .unwrap(),
        );
        let fxa_uid = rs.get_string_by_name("fxa_uid").unwrap().unwrap();
        let quantity = rs.get_i64_by_name("quantity").unwrap().unwrap();
        let plan_id = rs.get_string_by_name("plan_id").unwrap().unwrap();
        let plan_currency = rs.get_string_by_name("plan_currency").unwrap().unwrap();
        let plan_amount = rs.get_i64_by_name("plan_amount").unwrap().unwrap();
        let country = rs.get_string_by_name("country").unwrap().unwrap();
        let promotion_codes = "";
        query_as!(
            Subscription,
            r#"INSERT INTO subscription (id, report_timestamp, subscription_start_date, fxa_uid, quantity, plan_id, plan_currency, plan_amount, country, promotion_codes)
			VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
			RETURNING *"#,
            id,
            report_timestamp,
			subscription_start_date,
			fxa_uid,
			i32::try_from(quantity).ok(),
			plan_id,
			plan_currency,
			i32::try_from(plan_amount).ok(),
			country,
			promotion_codes
        )
        .fetch_one(self.db_pool)
        .await
    }
}
