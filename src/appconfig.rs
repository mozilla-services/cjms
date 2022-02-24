use crate::handlers;
use actix_web::{
    dev::Server,
    middleware,
    web::{get, resource, scope, ServiceConfig},
    App, HttpServer,
};
use std::net::TcpListener;

fn config_app(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/")
            .service(resource("").route(get().to(handlers::index)))
            .service(resource("__heartbeat__").route(get().to(handlers::heartbeat)))
            .service(resource("__lbheartbeat__").route(get().to(handlers::heartbeat))),
    );
}

pub fn run_server(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .configure(config_app)
            .wrap(middleware::Logger::default())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
