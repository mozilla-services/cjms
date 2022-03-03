use actix_web::{
    dev::Server,
    web::{get, post, put, resource, Data},
    App, HttpServer,
};
use sqlx::{migrate, PgPool};
use std::net::TcpListener;

use crate::controllers;

pub fn run_server(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .service(resource("/").route(get().to(controllers::custodial::index)))
            .service(resource("/__heartbeat__").route(get().to(controllers::custodial::heartbeat)))
            .service(
                resource("/__lbheartbeat__").route(get().to(controllers::custodial::heartbeat)),
            )
            .service(resource("/__version__").route(get().to(controllers::custodial::version)))
            .service(resource("/aic").route(post().to(controllers::aic::create)))
            .service(resource("/aic/{aic_id}").route(put().to(controllers::aic::update)))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}

pub async fn connect_to_database_and_migrate(database_url: &str) -> PgPool {
    let connection_pool = PgPool::connect(database_url)
        .await
        .expect("Failed to connect to Postgres.");
    migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database.");
    connection_pool
}
