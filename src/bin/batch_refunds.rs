use lib::{appconfig::CJ, jobs::batch_refunds::batch_refunds_by_day, telemetry::TraceType};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cj = CJ::new(TraceType::BatchRefunds).await;
    batch_refunds_by_day(&cj.db_pool, &cj.statsd).await;
    cj.shutdown().await
}
