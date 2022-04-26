use lib::{
    appconfig::{run_server, CJ},
    info,
    telemetry::TraceType,
};
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cj = CJ::new(TraceType::WebApp).await;
    let addr = cj.settings.server_address();
    info!(
        TraceType::WebApp,
        addr = format!("http://{}", addr).as_str(),
        "Server running"
    );
    run_server(
        cj.settings.clone(),
        TcpListener::bind(addr)?,
        cj.db_pool.clone(),
    )?
    .await?;
    cj.shutdown().await
}
