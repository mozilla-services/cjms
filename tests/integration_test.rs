use actix_web::{App, HttpServer};
use cjms::appconfig::config_app;
use cjms::settings::{get_settings, Settings};
use std::env;
use std::net::TcpListener;

pub struct TestApp {
    pub settings: Settings,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    // RM let address = format!("http://127.0.0.1:{}", port);
    env::set_var("HOST", "127.0.0.1");
    env::set_var("PORT", format!("{}", port));
    let settings = get_settings(None);
    let server = HttpServer::new(|| App::new().configure(config_app))
        .listen(listener)
        .expect("Server could not be configured")
        .run();
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
    //assert_eq!(response.bytes(), Bytes::from_static(b"Hellow world!"));
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
