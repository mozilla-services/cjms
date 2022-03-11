use crate::actions::bigquery::lib::get_bq_results;
use crate::models::subscription::SubscriptionModel;
use crate::settings::Settings;
use sqlx::PgPool;

pub async fn check_subscriptions(
    pool: &PgPool,
    big_query_access_token: String,
    settings: Settings,
) {
    let project = settings.gcp_project;
    let query =
        "SELECT * FROM `cjms_bigquery.sarah_test` WHERE report_timestamp IS NOT NULL LIMIT 3";
    let mut rs = get_bq_results(&big_query_access_token, query, &project).await;
    let subs = SubscriptionModel { db_pool: pool };
    while rs.next_row() {
        let subscription = subs.create_from_big_query_resultset(&rs).await;
        println!("subscription: {:?}", subscription);
    }
}
