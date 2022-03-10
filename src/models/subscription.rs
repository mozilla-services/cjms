use fake::{Fake, StringFaker};
use rand::seq::SliceRandom;
use rand::Rng;
use sqlx::{query_as, Error, PgPool};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

pub fn random_ascii_string() -> String {
    const ASCII: &str =
        "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&\'()*+,-./:;<=>?@";
    let f = StringFaker::with(Vec::from(ASCII), 8..32);
    f.fake()
}

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
    pub async fn create(&self) -> Result<Subscription, Error> {
        let mut rng = rand::thread_rng();
        let id = Uuid::new_v4();
        let report_timestamp = OffsetDateTime::now_utc();
        let subscription_start_date = report_timestamp - Duration::hours(rng.gen_range(1..48));
        let fxa_uid = random_ascii_string();
        let quantity = 1;
        let plan_id = random_ascii_string();
        let currencies = vec!["USD", "GBP", "EUR"];
        let plan_currency = currencies.choose(&mut rand::thread_rng());
        let plan_amount = rng.gen_range(99..1099);
        let countries = vec!["US", "DE", "FR"];
        let country = countries.choose(&mut rand::thread_rng());
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
			quantity,
			plan_id,
			plan_currency,
			plan_amount,
			country,
			promotion_codes
        )
        .fetch_one(self.db_pool)
        .await
    }
}
