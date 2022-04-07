use lib::{
    appconfig::{connect_to_database_and_migrate, run_server},
    settings::get_settings,
    telemetry::{init_sentry, init_tracing, StatsD, TraceType},
};
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = get_settings();

    let _guard = init_sentry(&settings);
    init_tracing("cjms-web", &settings.log_level, std::io::stdout);
    let statsd = StatsD::new(&settings);
    statsd.incr(TraceType::WebAppInit);

    let addr = settings.server_address();
    let db_pool = connect_to_database_and_migrate(&settings.database_url).await;
    tracing::info!(r#type = "server-init", "Server running at http://{}", addr);
    run_server(settings, TcpListener::bind(addr)?, db_pool)?.await?;

    Ok(())
}
