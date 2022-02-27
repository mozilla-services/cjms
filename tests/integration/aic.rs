use cjms::{controllers::aic::AICResponse, models::aic::AICModel};
use serde_json::json;
use time::OffsetDateTime;
use uuid::Version;

use crate::utils::{random_ascii_string, spawn_app};

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
async fn aic_create_success() {
    /* Caller sends flowId and CJEvent value and not an AIC value
        - create a new AIC id
        - save creation time, expiration time, AIC id, flow ID, CJ event value
        - return expiration time, and AIC id
    */

    /* SETUP */
    let app = spawn_app().await;
    let cj_event_value = random_ascii_string();
    let flow_id = random_ascii_string();
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
    let response: AICResponse = r.json().await.expect("Failed to get JSON response.");

    /* TEST */

    // Should be UUID v4 aka Version::Random
    let returned_uuid = response.aic_id;
    assert_eq!(Some(Version::Random), returned_uuid.get_version());

    /*
    Expires date is 30 days from today
    (because we created the expires a few nano seconds a go, this is a minute under 30 days)
    */
    assert_eq!(
        (response.expires - OffsetDateTime::now_utc()).whole_minutes(),
        30 * 24 * 60 - 1
    );

    let model = AICModel {
        db_pool: &app.db_connection(),
    };
    let saved = model.fetch_one().await.expect("Failed to get DB response.");
    assert_eq!(saved.id, response.aic_id);
    assert_eq!(
        (saved.created - OffsetDateTime::now_utc()).whole_minutes(),
        0
    );
    /*
    When serde serializes, it uses the unix timestamp so response.expires is missing
    the nanoseconds that are stored in the database.
    */
    assert_eq!(
        saved.expires.unix_timestamp(),
        response.expires.unix_timestamp()
    );
    assert_eq!(saved.cj_event_value, cj_event_value);
    assert_eq!(saved.flow_id, flow_id);
}

#[tokio::test]
async fn aic_create_with_bad_data() {
    let app = spawn_app().await;
    let test_cases = [
        json!({
            "flow_id": random_ascii_string(),
            "cj_id": 42,
        }),
        json!({
            "flow_id": 42,
            "cj_id": random_ascii_string(),
        }),
    ];
    for data in test_cases {
        let path = app.build_url("/aic");
        let client = reqwest::Client::new();
        let r = client
            .post(&path)
            .json(&data)
            .send()
            .await
            .expect("Failed to POST");
        assert_eq!(r.status(), 400);
        let response = r.text().await.expect("Failed to get response text.");
        assert!(response.contains("Json deserialize error"));
    }
}

#[tokio::test]
async fn aic_update_with_existing_aic_and_new_flow_and_cjid() {
    /* Caller sends AIC id, flowId, new CJEvent value
        - keep existing AIC id
        - save new creation time, new expiration time, new flow ID, new CJ event value
        - return new expiration time, existing AIC id
    */

    /* SETUP */
    let app = spawn_app().await;
    let cj_event_value_orig = random_ascii_string();
    let flow_id_orig = random_ascii_string();
    let model = AICModel {
        db_pool: &app.db_connection(),
    };
    let aic_orig = model
        .create(&cj_event_value_orig, &flow_id_orig)
        .await
        .expect("Failed to create test object.");
    std::thread::sleep(std::time::Duration::from_secs(1));
    let path = format!("/aic/{}", aic_orig.id);

    let cj_event_value_new = format!("{}{}", cj_event_value_orig, "extra");
    let flow_id_new = format!("{}{}", flow_id_orig, "extra");
    let update_data = json!({
        "cj_id": cj_event_value_new,
        "flow_id": flow_id_new,
    });

    /* CALL */
    let path = app.build_url(&path);
    let client = reqwest::Client::new();
    let r = client
        .put(&path)
        .json(&update_data)
        .send()
        .await
        .expect("Failed to PUT");
    assert_eq!(r.status(), 201);
    let response: AICResponse = r.json().await.expect("Failed to get JSON response.");

    /* TEST */
    assert_eq!(aic_orig.id, response.aic_id);
    // New expires time should be later than the original
    assert!(response.expires > aic_orig.expires);
    let saved = model.fetch_one().await.expect("Failed to get DB response.");
    assert_eq!(saved.id, response.aic_id);
    assert_eq!(
        saved.expires.unix_timestamp(),
        response.expires.unix_timestamp()
    );
    assert_eq!(saved.cj_event_value, cj_event_value_new);
    assert_eq!(saved.flow_id, flow_id_new);
}

#[tokio::test]
async fn aic_update_when_aic_and_cjevent_exists() {
    /* Caller sends AIC id, flowId, existing CJEvent value
        - keep existing AIC id, creation time, expiration time, cjevent value
        - save new flow ID
        - return existing expiration time, existing AIC id
    */
    /* SETUP */
    let app = spawn_app().await;
    let cj_event_value_orig = random_ascii_string();
    let flow_id_orig = random_ascii_string();
    let model = AICModel {
        db_pool: &app.db_connection(),
    };
    let aic_orig = model
        .create(&cj_event_value_orig, &flow_id_orig)
        .await
        .expect("Failed to create test object.");
    std::thread::sleep(std::time::Duration::from_secs(1));
    let path = format!("/aic/{}", aic_orig.id);

    let flow_id_new = format!("{}{}", flow_id_orig, "extra");
    let update_data = json!({
        "cj_id": cj_event_value_orig,
        "flow_id": flow_id_new,
    });

    /* CALL */
    let path = app.build_url(&path);
    let client = reqwest::Client::new();
    let r = client
        .put(&path)
        .json(&update_data)
        .send()
        .await
        .expect("Failed to PUT");
    assert_eq!(r.status(), 201);
    let response: AICResponse = r.json().await.expect("Failed to get JSON response.");

    /* TEST */
    assert_eq!(aic_orig.id, response.aic_id);
    // New expires time should be later than the original
    assert_eq!(
        response.expires.unix_timestamp(),
        aic_orig.expires.unix_timestamp()
    );
    let saved = model.fetch_one().await.expect("Failed to get DB response.");
    assert_eq!(saved.id, response.aic_id);
    assert_eq!(
        saved.expires.unix_timestamp(),
        response.expires.unix_timestamp()
    );
    assert_eq!(saved.cj_event_value, cj_event_value_orig);
    assert_eq!(saved.flow_id, flow_id_new);
}

/*

#[actix_rt::test]
async fn aic_endpoint_when_no_aic_exists() {
    /* Caller sends flowId, CJEvent value, and AIC value but AIC doesn't exist in our DB
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



*/
