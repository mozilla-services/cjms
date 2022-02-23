use actix_web::web::{get, resource, scope, ServiceConfig};

use crate::handlers;

pub fn config_app(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/")
            .service(resource("").route(get().to(handlers::index)))
            .service(resource("__heartbeat__").route(get().to(handlers::heartbeat)))
            .service(resource("__lbheartbeat__").route(get().to(handlers::heartbeat))),
    );
}
