use actix_web::{middleware, App, HttpServer};

use cjms::appconfig::config_app;
use cjms::env::get_env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let env = get_env();
    let addr = format!("{}:{}", env.host, env.port);
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
