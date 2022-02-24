use cjms::appconfig::run_server;
use cjms::settings::{get_settings, Settings};
use std::env;
use std::net::TcpListener;

pub struct TestApp {
    pub settings: Settings,
}

async fn spawn_app() -> TestApp {
    let host = "127.0.0.1";
    let listener = TcpListener::bind(format!("{}:0", host)).expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    env::set_var("HOST", host.to_string());
    env::set_var("PORT", format!("{}", port));
    let settings = get_settings(None);
    let server = run_server(listener).expect("Failed to start server");
    let _ = tokio::spawn(server);
    TestApp { settings }
}

fn build_url(app: &TestApp, path: &str) -> String {
    format!("http://{}{}", app.settings.server_address(), path)
}

#[tokio::test]
async fn test_index_get() {
    let app = spawn_app().await;
    let path = build_url(&app, "/");
    let r = reqwest::get(path).await.expect("Failed to execute request");
    assert_eq!(r.status(), 200);
    let body = r.text().await.expect("Response body missing.");
    assert_eq!(body, "Hello world!");
}

#[tokio::test]
async fn test_heartbeats_get() {
    let app = spawn_app().await;
    let test_cases = vec!["/__heartbeat__", "/__lbheartbeat__"];
    for path in test_cases {
        let path = build_url(&app, path);
        let r = reqwest::get(&path)
            .await
            .expect("Failed to execute request");
        assert_eq!(r.status(), 200, "Failed on path: {}", path);
    }
}
