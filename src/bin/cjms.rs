use actix_web::{middleware, web, App, HttpServer};
use dotenv::dotenv;

use cjms::appconfig::config_app;
use cjms::db::create_database_pool;
use cjms::env::get_env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let env = get_env();

    let db_pool = create_database_pool(&env.database_url);

    let addr = format!("{}:{}", env.host, env.port);
    println!("Server running at http://{}", addr);
    HttpServer::new(move || {
        App::new()
            .configure(config_app)
            .app_data(web::Data::new(db_pool.clone()))
            .wrap(middleware::Logger::default())
    })
    .bind(addr)?
    .run()
    .await
}
