use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use time::{Date, OffsetDateTime};

use crate::{
    models::{
        refunds::{Refund, RefundModel},
        subscriptions::SubscriptionModel,
    },
    settings::Settings,
};

async fn build_body_from_results(
    settings: &Settings,
    results: Vec<Refund>,
    db_pool: &PgPool,
) -> String {
    let mut body = format!(
        r#"
&CID={}
&SUBID={}"#,
        settings.cj_cid, settings.cj_subid
    );
    let subscriptions = SubscriptionModel { db_pool };
    for refund in results {
        let sub = match subscriptions
            .fetch_one_by_subscription_id(&refund.subscription_id)
            .await
        {
            Ok(sub) => {
                // TODO - LOGGING
                println!(
                    "Success fetching sub {} for refund {}",
                    refund.subscription_id, refund.refund_id
                );
                sub
            }
            Err(_) => {
                println!(
                    "Failed to fetch sub {} for refund {}. Continuing....",
                    refund.subscription_id, refund.refund_id
                );
                continue;
            }
        };
        body.push_str(&format!(
            r#"
RETRN,,{}"#,
            sub.id
        ));
    }
    body
}

async fn get_results_for_day(db_pool: &PgPool, day: Date) -> Vec<Refund> {
    let refunds = RefundModel { db_pool };
    // Intentional panic, can't continue if can't get refunds for today
    refunds
        .fetch_by_correction_file_day(&day)
        .await
        .unwrap_or_else(|_| panic!("Could not fetch refunds for date: {}", day))
}

#[derive(Deserialize)]
pub struct CorrectionsByDayPath {
    #[serde(with = "date_parser")]
    day: Date,
}

mod date_parser {
    use serde::{self, Deserialize, Deserializer};
    use time::Date;
    const FORMAT: &str = "%F";
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Date::parse(s, FORMAT).map_err(serde::de::Error::custom)
    }
}

pub async fn by_day(
    path: web::Path<CorrectionsByDayPath>,
    pool: web::Data<PgPool>,
    settings: web::Data<Settings>,
) -> HttpResponse {
    let results = get_results_for_day(pool.as_ref(), path.day).await;
    let body = build_body_from_results(settings.as_ref(), results, pool.as_ref()).await;
    HttpResponse::Ok().body(body)
}

pub async fn today(pool: web::Data<PgPool>, settings: web::Data<Settings>) -> HttpResponse {
    // TODO - LOGGING - Add statsd metrics to see how often this is running
    let today = OffsetDateTime::now_utc().date();
    let results = get_results_for_day(pool.as_ref(), today).await;
    let body = build_body_from_results(settings.as_ref(), results, pool.as_ref()).await;
    HttpResponse::Ok().body(body)
}
