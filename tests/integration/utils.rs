use cjms::appconfig::{connect_to_database_and_migrate, run_server};
use cjms::settings::{get_settings, Settings};
use sqlx::{Connection, Executor, PgConnection};
use std::net::TcpListener;
use uuid::Uuid;

pub struct TestApp {
    pub settings: Settings,
}

async fn create_test_database(database_url: &str) -> String {
    let randomized_test_database_url = format!("{}_test_{}", database_url, Uuid::new_v4());
    let url_parts: Vec<&str> = randomized_test_database_url.rsplit('/').collect();
    let database_name = url_parts.get(0).unwrap().to_string();
    let postgres_url =
        randomized_test_database_url.replace(format!("/{}", &database_name).as_str(), "");
    let mut connection = PgConnection::connect(&postgres_url)
        .await
        .expect("Failed to connect to postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, &database_name).as_str())
        .await
        .expect("Failed to create test database.");
    randomized_test_database_url
}

pub async fn spawn_app() -> TestApp {
    let mut settings = get_settings();
    let listener =
        TcpListener::bind(format!("{}:0", settings.host)).expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    settings.port = format!("{}", port);
    let test_database_url = create_test_database(&settings.database_url).await;
    let db_pool = connect_to_database_and_migrate(test_database_url).await;
    let server = run_server(listener, db_pool).expect("Failed to start server");
    let _ = tokio::spawn(server);
    TestApp { settings }
}

pub fn build_url(app: &TestApp, path: &str) -> String {
    format!("http://{}{}", app.settings.server_address(), path)
}
