use lib::{appconfig::CJ, jobs::cleanup::archive_expired_aics, telemetry::TraceType};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cj = CJ::new(TraceType::Cleanup).await;
    archive_expired_aics(&cj.db_pool).await;
    cj.shutdown().await
}
