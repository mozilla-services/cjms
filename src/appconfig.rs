use crate::handlers;
use actix_web::{dev::Server, web::get, App, HttpServer};
use std::net::TcpListener;

pub fn run_server(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/", get().to(handlers::index))
            .route("/__heartbeat__", get().to(handlers::heartbeat))
            .route("/__lbheartbeat__", get().to(handlers::heartbeat))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
