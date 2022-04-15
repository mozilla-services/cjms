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
use sentry::ClientInitGuard;
use sqlx::{migrate, PgPool};
use std::net::TcpListener;
use time::OffsetDateTime;
use tracing_actix_web_mozlog::MozLog;

use crate::{
    bigquery::client::{get_bqclient, BQClient},
    cj::client::CJS2SClient,
    controllers,
    settings::{get_settings, Settings},
    telemetry::{info, init_sentry, init_tracing, StatsD, TraceType},
};

pub struct CJ {
    _guard: ClientInitGuard,
    name: TraceType,
    start: OffsetDateTime,
    pub bq_client: BQClient,
    pub cj_client: CJS2SClient,
    pub db_pool: PgPool,
    pub settings: Settings,
    pub statsd: StatsD,
}

impl CJ {
    pub async fn new(name: TraceType) -> Self {
        let start = OffsetDateTime::now_utc();
        let settings = get_settings();
        let _guard = init_sentry(&settings);
        if name != TraceType::Test {
            init_tracing(&name.to_string(), &settings.log_level, std::io::stdout);
        }
        let db_pool = connect_to_database_and_migrate(&settings.database_url).await;
        let bq_client = get_bqclient(&settings).await;
        let cj_client = CJS2SClient::new(&settings, None);
        let statsd = StatsD::new(&settings);
        statsd.incr(&name, "starting");
        info(&name, "Starting");
        CJ {
            _guard,
            name,
            start,
            bq_client,
            cj_client,
            db_pool,
            settings,
            statsd,
        }
    }

    pub async fn shutdown(&self) -> std::io::Result<()> {
        self.statsd.incr(&self.name, "ending");
        info(&self.name, "Ending");
        self.db_pool.close().await;
        self.statsd
            .time(&self.name, "timer", OffsetDateTime::now_utc() - self.start);
        Ok(())
    }
}

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
    let server = HttpServer::new(move || {
        let db_pool_d = Data::new(db_pool.clone());
        let settings_d = Data::new(settings.clone());
        let statsd_d = Data::new(StatsD::new(&settings));
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
            .service(resource("/__error_log__").route(get().to(controllers::custodial::error_log)))
            .service(
                resource("/__error_panic__").route(get().to(controllers::custodial::error_panic)),
            )
            // AIC
            .service(resource("/aic").route(post().to(controllers::aic::create)))
            .service(resource("/aic/{aic_id}").route(put().to(controllers::aic::update)))
            // Corrections
            .service(
                resource("/corrections/today.csv").route(get().to(controllers::corrections::today)),
            )
            .service(
                resource("/corrections/{day}.csv")
                    .route(get().to(controllers::corrections::by_day))
                    .wrap(auth),
            )
            // Make data objects available to all routes
            .app_data(db_pool_d)
            .app_data(settings_d)
            .app_data(statsd_d)
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
