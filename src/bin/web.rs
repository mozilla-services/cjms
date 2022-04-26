use lib::{
    appconfig::{run_server, CJ},
    telemetry::{info, TraceType},
};
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cj = CJ::new(TraceType::WebApp).await;
    let addr = cj.settings.server_address();
    info(
        &TraceType::WebApp,
        &format!("Server running at http://{}", addr),
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
