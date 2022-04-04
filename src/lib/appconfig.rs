use actix_cors::Cors;
use actix_web::{
    dev::{Server, ServiceRequest},
    error::ErrorUnauthorized,
    http,
    web::{get, post, put, resource, Data},
    App, Error, HttpServer,
};
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web_httpauth::middleware::HttpAuthentication;
use sqlx::{migrate, PgPool};
use std::net::TcpListener;
use tracing_actix_web_mozlog::MozLog;

use crate::{controllers, settings::Settings};

async fn basic_auth_middleware(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, Error> {
    // Intentional expect. Can't go on without them.
    let settings = req.app_data::<Data<Settings>>().expect("Missing settings");
    let password = match credentials.password() {
        Some(password) => password,
        None => return Err(ErrorUnauthorized("Password missing.")),
    };
    if password.eq(&settings.authentication) {
        Ok(req)
    } else {
        Err(ErrorUnauthorized("Incorrect password."))
    }
}

pub fn run_server(
    settings: Settings,
    listener: TcpListener,
    db_pool: PgPool,
) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let server = HttpServer::new(move || {
        let cors = get_cors(settings.clone());
        let moz_log = MozLog::default();
        let auth = HttpAuthentication::basic(basic_auth_middleware);
        App::new()
            .wrap(moz_log)
            .wrap(cors)
            // Custodial
            .service(resource("/").route(get().to(controllers::custodial::index)))
            .service(resource("/__heartbeat__").route(get().to(controllers::custodial::heartbeat)))
            .service(
                resource("/__lbheartbeat__").route(get().to(controllers::custodial::heartbeat)),
            )
            .service(resource("/__version__").route(get().to(controllers::custodial::version)))
            // AIC
            .service(resource("/aic").route(post().to(controllers::aic::create)))
            .service(resource("/aic/{aic_id}").route(put().to(controllers::aic::update)))
            // Corrections
            .service(
                resource("/corrections/today.csv")
                    .route(get().to(controllers::corrections::today))
                    .app_data(Data::new(settings.clone())),
            )
            .service(
                resource("/corrections/{day}.csv")
                    .route(get().to(controllers::corrections::by_day))
                    .wrap(auth)
                    .app_data(Data::new(settings.clone())),
            )
            // Make DB available to all routes
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

fn get_cors(settings: Settings) -> Cors {
    let mut cors = Cors::default()
        .allow_any_method()
        .allowed_headers(vec![http::header::ACCEPT, http::header::CONTENT_TYPE]);
    for origin in allowed_origins(&settings) {
        cors = cors.allowed_origin(origin);
    }
    cors
}

fn allowed_origins(settings: &Settings) -> Vec<&'static str> {
    let allowed = match settings.environment.as_str() {
        "prod" => {
            vec!["https://www.mozilla.org", "https://www.allizom.org"]
        }
        "local" | "dev" | "stage" => {
            vec![
                "http://localhost:8000",
                "https://www-dev.allizom.org",
                "https://www-demo1.allizom.org",
                "https://www-demo2.allizom.org",
                "https://www-demo3.allizom.org",
                "https://www-demo4.allizom.org",
                "https://www-demo5.allizom.org",
            ]
        }
        _ => panic!("Invalid settings value"),
    };
    allowed
}

#[cfg(test)]
mod test_appconfig {
    use super::*;
    use crate::test_utils::empty_settings;

    #[test]
    fn test_allowed_origins_for_stage_and_dev() {
        let mut settings = empty_settings();
        for test_case in ["local", "stage", "dev"] {
            settings.environment = test_case.to_string();
            let origins = allowed_origins(&settings);
            assert_eq!(origins.len(), 7);
            for expected_origin in [
                "http://localhost:8000",
                "https://www-dev.allizom.org",
                "https://www-demo1.allizom.org",
                "https://www-demo2.allizom.org",
                "https://www-demo3.allizom.org",
                "https://www-demo4.allizom.org",
                "https://www-demo5.allizom.org",
            ] {
                assert!(
                    origins.contains(&expected_origin),
                    "Didn't find: {} in {:?}",
                    expected_origin,
                    origins
                );
            }
        }
    }

    #[test]
    fn test_allowed_origins_for_prod() {
        let mut settings = empty_settings();
        settings.environment = "prod".to_string();
        let origins = allowed_origins(&settings);
        assert_eq!(origins.len(), 2);
        for expected_origin in ["https://www.mozilla.org", "https://www.allizom.org"] {
            assert!(origins.contains(&expected_origin));
        }
    }

    #[test]
    #[should_panic]
    fn test_allowed_origins_for_not_allowed() {
        allowed_origins(&empty_settings());
    }
}
