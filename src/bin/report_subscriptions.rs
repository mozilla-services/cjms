use lib::{
    appconfig::CJ, jobs::report_subscriptions::report_subscriptions_to_cj, telemetry::LogKey,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cj = CJ::new(LogKey::ReportSubscriptions).await;
    report_subscriptions_to_cj(&cj.db_pool, &cj.cj_client, &cj.statsd).await;
    cj.shutdown().await
}
