use sqlx::PgPool;

use crate::actions::bigquery::run_bq_table_get;
use crate::models::subscription::SubscriptionModel;

pub async fn check_subscriptions(pool: &PgPool, big_query_access_token: String) {
    // Manually run the code that gets the access token by workload identity to see what response we get

    let query = r#"
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
  LIMIT 3
                "#;
    let response = run_bq_table_get(big_query_access_token, query).await;
    let data = response.text().await.expect("Couldn't extract body.");
    println!("BQ response: {}", data);
    /*
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
    */
    let subs = SubscriptionModel { db_pool: pool };
    for _ in 0..3 {
        subs.create().await.expect("Create failed :(");
    }
}
