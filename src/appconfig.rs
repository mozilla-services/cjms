use crate::handlers;
use actix_web::{
    dev::Server,
    web::{get, post, put},
    App, HttpServer,
};
use std::net::TcpListener;

pub fn run_server(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/", get().to(handlers::index))
            .route("/__heartbeat__", get().to(handlers::heartbeat))
            .route("/__lbheartbeat__", get().to(handlers::heartbeat))
            .route("/aic", post().to(handlers::aic_create))
            .route("/aic/{aic_id}", put().to(handlers::aic_update))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
