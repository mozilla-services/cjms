//use chrono::{DateTime, Utc};
use cjms::appconfig::run_server;
//use cjms::handlers::{AICResponse};
use cjms::settings::{get_settings, Settings};
use std::env;
use std::net::TcpListener;
//use serde_json::json;
//use uuid::{Uuid, Version};

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

/*
    * START /aic endpoint (Affiliate Identifier Cookie)
    *

#[actix_rt::test]
async fn test_aic_get_is_not_allowed() {
    let mut app = setup_app!();
    let req = test::TestRequest::get().uri("/aic").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 405);
    let req = test::TestRequest::get().uri("/aic/123").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 405);
}

#[actix_rt::test]
async fn test_aic_endpoint_when_no_aic_sent() {
    /* Bedrock sends flowId and CJEvent value and not an AIC value
        - create a new AIC id
        - save creation time, expiration time, AIC id, flow ID, CJ event value
        - return expiration time, and AIC id
    */

    /* SETUP */
    let mut app = setup_app!();
    let cj_event_value = "123ABC";
    let flow_id = "4jasdrkl";
    let data = json!({
        "flow_id": flow_id,
        "cj_id": cj_event_value,
    });
    /* CALL */
    let req = test::TestRequest::post().set_json(&data).uri("/aic").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 201);
    let resp:AICResponse = test::read_body_json(resp).await;

    /* CHECK RESPONSE */
    // Should be UUID v4 aka Version::Random
    let returned_uuid = Uuid::parse_str(&resp.aic_id).unwrap();
    assert_eq!(Some(Version::Random), returned_uuid.get_version());
    // Expires date is 30 days from today
    // (because we created the expires a few nano seconds a go, this is a minute under 30 days)
    let expires = DateTime::parse_from_rfc2822(&resp.expires).unwrap();
    assert_eq!(expires.signed_duration_since(Utc::now()).num_minutes(), 30 * 24 * 60 - 1);

    /* CHECK DATABASE */
    assert!(false);
}

#[actix_rt::test]
async fn test_something_happens_when_wrong_data_is_sent() {
    assert_eq!(true, false);
}

#[actix_rt::test]
async fn test_aic_endpoint_when_no_aic_exists() {
    /* Bedrock sends flowId, CJEvent value, and AIC value but AIC doesn't exist in our DB
        - create a new AIC id
        - save creation time, expiration time, AIC id, flow ID, CJ event value
        - return expiration time, and AIC id
    */
    let mut app = setup_app!();
    let req = test::TestRequest::put().uri("/aic/123").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 201);
    assert_eq!(true, false);
}

#[actix_rt::test]
async fn test_aic_endpoint_when_aic_exists() {
    /* Bedrock sends AIC id, flowId, new CJEvent value
        - keep existing AIC id
        - save new creation time, new expiration time, new flow ID, new CJ event value
        - return new expiration time, existing AIC id
    */
    let mut app = setup_app!();
    let req = test::TestRequest::put().uri("/aic/123").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(true, false);
}

#[actix_rt::test]
async fn test_aic_endpoint_when_aic_and_cjevent_exists() {
    /* Bedrock sends AIC id, flowId, existing CJEvent value
        - keep existing AIC id, creation time, expiration time, cjevent value
        - save new flow ID
        - return existing expiration time, existing AIC id
    */
    let mut app = setup_app!();
    let req = test::TestRequest::put().uri("/aic/123").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(true, false);
}

// END /aic endpoint

*/
