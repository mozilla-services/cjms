use sqlx::PgPool;

use crate::actions::bigquery::lib::run_bq_table_get;
use crate::actions::bigquery::model::{GetQueryResultsResponse, QueryResponse, ResultSet};
use crate::models::subscription::SubscriptionModel;
use crate::settings::Settings;

pub async fn check_subscriptions(
    pool: &PgPool,
    big_query_access_token: String,
    settings: Settings,
) {
    // Manually run the code that gets the access token by workload identity to see what response we get
    let project = settings.gcp_project;
    let query = format!(
        "SELECT * FROM `{}.cjms_bigquery.sarah_test` LIMIT 3",
        &project
    );
    let response = run_bq_table_get(&big_query_access_token, &query, &project).await;
    let query_results: GetQueryResultsResponse =
        response.json().await.expect("Couldn't extract body.");
    let mut rs = ResultSet::new(QueryResponse::from(query_results));
    while rs.next_row() {
        let plan_id = rs.get_string_by_name("plan_id").expect("no plan_id");
        let start = rs
            .get_i64_by_name("subscription_start_date")
            .expect("no start date");
        println!("plan_id: {:?} | subscription_start: {:?}", plan_id, start);
    }
    let subs = SubscriptionModel { db_pool: pool };
    for _ in 0..3 {
        subs.create().await.expect("Create failed :(");
    }
}
