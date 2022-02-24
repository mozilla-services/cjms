use cjms::appconfig::run_server;
use cjms::settings::{get_settings, Settings};
use std::net::TcpListener;
use std::env;

pub struct TestApp {
    pub settings: Settings,
}

async fn spawn_app() -> TestApp {
    let host = "127.0.0.1";
    let listener = TcpListener::bind(format!("{}:0", host)).expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    env::set_var("HOST", format!("{}", host));
    env::set_var("PORT", format!("{}", port));
    let settings = get_settings(None);
    let server = run_server(settings.server_address()).expect("Failed to start server");
    let _ = tokio::spawn(server);
    TestApp { settings }
}

fn build_url(app: TestApp, path: &str) -> String {
    format!("http://{}{}", app.settings.server_address(), path)
}

#[tokio::test]
async fn test_index_get() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let response = client
        .get(build_url(app, "/"))
        .send()
        .await
        .expect("Failed to execute request");
    assert!(response.status().is_success());
    let body = response.text().await.expect("Response body missing.");
    assert_eq!(body, "Hello world!");
}
/*
#[tokio::test]
async fn test_heartbeat_get() {
    let mut app = setup_app!();
    let req = test::TestRequest::get().uri("/__heartbeat__").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_lbheartbeat_get() {
    let mut app = setup_app!();
    let req = test::TestRequest::get()
        .uri("/__lbheartbeat__")
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 200);
}
 */
