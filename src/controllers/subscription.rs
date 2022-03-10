use actix_web::{web, Error, HttpResponse};
use gcp_bigquery_client::{model::query_request::QueryRequest, Client};
use sqlx::PgPool;

use crate::models::subscription::SubscriptionModel;

pub async fn check_subscriptions(pool: &PgPool) {
    let client = Client::with_workload_identity(true)
        .await
        .expect("Could not connect to BigQuery with workload identity");
    let rs = client
        .job()
        .query(
            "mozdata",
            QueryRequest::new(
                r#"
SELECT
  CURRENT_TIMESTAMP AS report_timestamp,
  subscription_start_date,
  subscription_id, -- transaction id
  fxa_uid,
  1 AS quantity,
  plan_id, -- sku
  plan_currency,
  plan_amount,
  country,
  promotion_codes,
  -- aic -- not available yet
  FROM `mozdata.mozilla_vpn.all_subscriptions`
  WHERE
  -- Exclude IAP providers
  provider NOT IN ("Apple Store", "Google Play")
  ORDER BY subscription_start_date DESC
  LIMIT 10
                "#,
            ),
        )
        .await;
    match rs {
        Ok(mut result) => {
            while result.next_row() {
                let plan_id = result.get_string_by_name("plan_id").expect("no plan_id");
                let start = result
                    .get_i64_by_name("subscription_start_date")
                    .expect("no start date");
                println!("plan_id: {:?} | subscription_start: {:?}", plan_id, start);
            }
        }
        Err(e) => {
            println!("Failed to connect to bq: {:?}", e);
        }
    }
    let subs = SubscriptionModel { db_pool: pool };
    for _ in 0..3 {
        subs.create().await.expect("Create failed :(");
    }
}

pub async fn check(_pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    // LEAVE THIS A NO-OP UNTIL WE CAN PUT IT BEHIND AUTH
    //check_subscriptions(pool.as_ref()).await;
    Ok(HttpResponse::Ok().body("Check subscriptions"))
}
