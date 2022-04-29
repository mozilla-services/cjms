use lib::{appconfig::CJ, jobs::verify_reports::verify_reports_with_cj, telemetry::LogKey};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cj = CJ::new(LogKey::VerifyReports).await;
    verify_reports_with_cj(&cj.db_pool, &cj.cj_client, &cj.statsd).await;
    cj.shutdown().await
}
