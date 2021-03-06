use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use time::{Date, OffsetDateTime};

use crate::{
    error_and_incr, info_and_incr,
    models::{
        refunds::{Refund, RefundModel},
        subscriptions::SubscriptionModel,
    },
    settings::Settings,
    telemetry::{LogKey, StatsD},
};

async fn build_body_from_results(
    settings: &Settings,
    results: Vec<Refund>,
    db_pool: &PgPool,
    statsd: &StatsD,
) -> String {
    let mut body = format!(
        r#"&CID={}
&SUBID={}"#,
        settings.cj_sftp_user, settings.cj_subid
    );
    let subscriptions = SubscriptionModel { db_pool };
    for refund in results {
        let sub = match subscriptions
            .fetch_one_by_subscription_id(&refund.subscription_id)
            .await
        {
            Ok(sub) => {
                info_and_incr!(
                    statsd,
                    LogKey::CorrectionsSubscriptionFetch,
                    subscription_id = refund.subscription_id.as_str(),
                    refund_id = refund.refund_id.as_str(),
                    "Success fetching sub for refund"
                );
                sub
            }
            Err(e) => {
                error_and_incr!(
                    statsd,
                    LogKey::CorrectionsSubscriptionFetchFailed,
                    error = e,
                    subscription_id = refund.subscription_id.as_str(),
                    refund_id = refund.refund_id.as_str(),
                    "Failed to fetch sub for refund. Continuing..."
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
    statsd: web::Data<StatsD>,
) -> HttpResponse {
    info_and_incr!(
        statsd.as_ref(),
        LogKey::CorrectionsReportByDayAccessed,
        day = path.day.to_string().as_str(),
        "Corrections report accessed by day"
    );
    let results = get_results_for_day(pool.as_ref(), path.day).await;
    let body =
        build_body_from_results(settings.as_ref(), results, pool.as_ref(), statsd.as_ref()).await;
    HttpResponse::Ok().body(body)
}

pub async fn today(
    pool: web::Data<PgPool>,
    settings: web::Data<Settings>,
    statsd: web::Data<StatsD>,
) -> HttpResponse {
    info_and_incr!(
        statsd.as_ref(),
        LogKey::CorrectionsReportTodayAccessed,
        "Corrections report accessed for today"
    );
    let today = OffsetDateTime::now_utc().date();
    let results = get_results_for_day(pool.as_ref(), today).await;
    let body =
        build_body_from_results(settings.as_ref(), results, pool.as_ref(), statsd.as_ref()).await;
    HttpResponse::Ok().body(body)
}
