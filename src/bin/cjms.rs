use actix_web::{middleware, App, HttpServer};

use cjms::appconfig::config_app;
use cjms::settings::get_settings;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = get_settings(None);
    let addr = settings.server_address();
    println!("Server running at http://{}", addr);
    HttpServer::new(|| {
        App::new()
            .configure(config_app)
            .wrap(middleware::Logger::default())
    })
    .bind(addr)?
    .run()
    .await
}
