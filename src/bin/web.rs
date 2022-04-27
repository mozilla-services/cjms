use lib::{
    appconfig::{run_server, CJ},
    info,
    telemetry::LogKey,
};
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cj = CJ::new(LogKey::WebApp).await;
    let addr = cj.settings.server_address();
    info!(
        LogKey::WebApp,
        addr = format!("http://{}", addr).as_str(),
        "Server running"
    );
    run_server(
        cj.settings.clone(),
        TcpListener::bind(addr)?,
        cj.db_pool.clone(),
        cj.statsd.clone(),
    )?
    .await?;
    cj.shutdown().await
}
