use lib::{appconfig::CJ, jobs::verify_reports::verify_reports_with_cj, telemetry::TraceType};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cj = CJ::new(TraceType::VerifyReports).await;
    verify_reports_with_cj(&cj.db_pool, &cj.cj_client, &cj.statsd).await;
    cj.shutdown().await
}
