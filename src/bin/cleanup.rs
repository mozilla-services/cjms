use lib::{appconfig::CJ, jobs::cleanup::archive_expired_aics, telemetry::LogKey};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cj = CJ::new(LogKey::Cleanup).await;
    archive_expired_aics(&cj.db_pool).await;
    cj.shutdown().await
}
