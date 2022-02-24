use crate::handlers;
use actix_web::{
    dev::Server,
    web::{get, post, put, resource},
    App, HttpServer,
};
use std::net::TcpListener;

pub fn run_server(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .service(resource("/").route(get().to(handlers::index)))
            .service(resource("/__heartbeat__").route(get().to(handlers::heartbeat)))
            .service(resource("/__lbheartbeat__").route(get().to(handlers::heartbeat)))
            .service(resource("/aic").route(post().to(handlers::aic_create)))
            .service(resource("/aic/{aic_id}").route(put().to(handlers::aic_update)))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
