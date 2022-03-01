use cjms::appconfig::{connect_to_database_and_migrate, run_server};
use cjms::settings::get_settings;
use std::net::TcpListener;
use env_logger::Env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // TODO this is pulled from The Book. Relies on RUST_LOG environment variable. should we use this directly or integrate it into settings.yaml?
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let settings = get_settings();
    let addr = settings.server_address();
    let db_pool = connect_to_database_and_migrate(&settings.database_url).await;
    println!("Server running at http://{}", addr);
    run_server(TcpListener::bind(addr)?, db_pool)?.await?;
    Ok(())
}
