use cjms::handlers::AICResponse;
use serde_json::json;
use time::OffsetDateTime;
use uuid::{Uuid, Version};

use crate::utils::spawn_app;

#[tokio::test]
async fn test_aic_get_is_not_allowed() {
    let app = spawn_app().await;
    let test_cases = vec!["/aic", "/aic/123"];
    for path in test_cases {
        let path = app.build_url(path);
        let r = reqwest::get(&path)
            .await
            .expect("Failed to execute request");
        assert_eq!(r.status(), 405, "Failed on path: {}", path);
    }
}

#[tokio::test]
async fn aic_endpoint_when_no_aic_sent() {
    /* Bedrock sends flowId and CJEvent value and not an AIC value
        - create a new AIC id
        - save creation time, expiration time, AIC id, flow ID, CJ event value
        - return expiration time, and AIC id
    */

    /* SETUP */
    let app = spawn_app().await;
    let cj_event_value = "123ABC";
    let flow_id = "4jasdrkl";
    let data = json!({
        "flow_id": flow_id,
        "cj_id": cj_event_value,
    });

    /* CALL */
    let path = app.build_url("/aic");
    let client = reqwest::Client::new();
    let r = client
        .post(&path)
        .json(&data)
        .send()
        .await
        .expect("Failed to POST");
    assert_eq!(r.status(), 201);
    let resp: AICResponse = r.json().await.expect("Failed to get JSON response.");

    /* CHECK RESPONSE */
    // Should be UUID v4 aka Version::Random
    let returned_uuid = Uuid::parse_str(&resp.aic_id).unwrap();
    assert_eq!(Some(Version::Random), returned_uuid.get_version());
    // Expires date is 30 days from today
    // (because we created the expires a few nano seconds a go, this is a minute under 30 days)
    assert_eq!(
        (resp.expires - OffsetDateTime::now_utc()).whole_minutes(),
        30 * 24 * 60 - 1
    );

    /* CHECK DATABASE */
    let saved = sqlx::query!("SELECT * FROM aic",)
        .fetch_one(&app.connection_pool())
        .await
        .expect("Failed to fetch saved aic.");
    assert_eq!(saved.id.to_string(), resp.aic_id);
    assert_eq!(saved.cj_event_value, "le guin");
}

/*
#[actix_rt::test]
async fn something_happens_when_wrong_data_is_sent() {
    assert_eq!(true, false);
}

#[actix_rt::test]
async fn aic_endpoint_when_no_aic_exists() {
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
async fn aic_endpoint_when_aic_exists() {
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
async fn aic_endpoint_when_aic_and_cjevent_exists() {
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

*/
