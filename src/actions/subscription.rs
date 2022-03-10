use sqlx::PgPool;

use crate::actions::bigquery::{run_bq_table_get, GetQueryResultsResponse};
use crate::models::subscription::SubscriptionModel;

pub async fn check_subscriptions(pool: &PgPool, big_query_access_token: String) {
    // Manually run the code that gets the access token by workload identity to see what response we get

    let query = r#"
SELECT
  *
  FROM `moz-fx-cjms-nonprod-9a36.cjms_bigquery.sarah_test`
  LIMIT 3
                "#;
    let response = run_bq_table_get(big_query_access_token, query).await;
    let data: GetQueryResultsResponse = response.json().await.expect("Couldn't extract body.");
    println!("BQ response: {:?}", data);
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
