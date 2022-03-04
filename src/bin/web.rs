use cjms::appconfig::{connect_to_database_and_migrate, run_server};
use cjms::settings::get_settings;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = get_settings();
    let addr = settings.server_address();
    let db_pool = connect_to_database_and_migrate(&settings.database_url).await;
    println!("Server running at http://{}", addr);
    run_server(TcpListener::bind(addr)?, db_pool)?.await?;
    Ok(())
}
