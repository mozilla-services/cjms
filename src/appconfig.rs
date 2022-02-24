use actix_web::dev::Server;
use actix_web::{middleware, App, HttpServer};

use actix_web::web::{get, resource, scope, ServiceConfig};

use crate::handlers;

fn config_app(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/")
            .service(resource("").route(get().to(handlers::index)))
            .service(resource("__heartbeat__").route(get().to(handlers::heartbeat)))
            .service(resource("__lbheartbeat__").route(get().to(handlers::heartbeat))),
    );
}


pub fn run_server(addr: String) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .configure(config_app)
            .wrap(middleware::Logger::default())
    })
    .bind(addr)
    .expect("Server could not be configured.")
    .run();
    Ok(server)
}
