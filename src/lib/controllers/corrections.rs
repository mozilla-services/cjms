use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use time::{Date, OffsetDateTime};

use crate::{
    models::refunds::{Refund, RefundModel},
    settings::Settings,
};

fn build_body_from_results(settings: &Settings, results: Vec<Refund>) -> String {
    let mut body = format!(
        r#"
&CID={}
&SUBID=123"#,
        settings.cj_cid
    );
    for refund in results {
        body.push_str(&format!(
            r#"
RETRN,,{}"#,
            refund.subscription_id
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
    let body = build_body_from_results(settings.as_ref(), results);
    HttpResponse::Ok().body(body)
}

pub async fn today(pool: web::Data<PgPool>, settings: web::Data<Settings>) -> HttpResponse {
    // TODO - LOGGING - Add statsd metrics to see how often this is running
    let today = OffsetDateTime::now_utc().date();
    let results = get_results_for_day(pool.as_ref(), today).await;
    let body = build_body_from_results(settings.as_ref(), results);
    HttpResponse::Ok().body(body)
}
