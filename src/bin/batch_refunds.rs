use lib::{appconfig::CJ, jobs::batch_refunds::batch_refunds_by_day, telemetry::LogKey};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cj = CJ::new(LogKey::BatchRefunds).await;
    batch_refunds_by_day(&cj.db_pool, &cj.statsd).await;
    cj.shutdown().await
}
