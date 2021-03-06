use lib::{
    appconfig::CJ, jobs::check_subscriptions::fetch_and_process_new_subscriptions,
    telemetry::LogKey,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cj = CJ::new(LogKey::CheckSubscriptions).await;
    fetch_and_process_new_subscriptions(&cj.bq_client, &cj.db_pool, &cj.statsd).await;
    cj.shutdown().await
}
