use lib::{appconfig::CJ, jobs::check_refunds::fetch_and_process_refunds, telemetry::LogKey};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cj = CJ::new(LogKey::CheckRefunds).await;
    fetch_and_process_refunds(&cj.bq_client, &cj.db_pool, &cj.statsd).await;
    cj.shutdown().await
}
