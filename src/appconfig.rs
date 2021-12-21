use actix_web::web;

use crate::handlers;

pub fn config_app(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/")
            .service(
                web::resource("")
                    .route(web::get().to(handlers::index))
            )
    );
}
