use lib::{
    appconfig::CJ, jobs::report_subscriptions::report_subscriptions_to_cj, telemetry::TraceType,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cj = CJ::new(TraceType::ReportSubscriptions).await;
    report_subscriptions_to_cj(&cj.db_pool, &cj.cj_client).await;
    cj.shutdown().await
}
