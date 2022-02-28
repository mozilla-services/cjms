use cjms::{
    controllers::aic::AICResponse,
    models::aic::{AICModel, AIC},
};
use serde_json::json;
use time::OffsetDateTime;
use uuid::{Uuid, Version};

use crate::utils::{
    random_ascii_string, send_get_request, send_post_request, send_put_request, spawn_app, TestApp,
};

#[tokio::test]
async fn test_aic_get_is_not_allowed() {
    let app = spawn_app().await;
    let test_cases = ["/aic", "/aic/123"];
    for path in test_cases {
        let r = send_get_request(&app, path).await;
        assert_eq!(r.status(), 405, "Failed on path: {}", path);
    }
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
        let r = send_post_request(&app, "/aic", data).await;
        assert_eq!(r.status(), 400);
        let response = r.text().await.expect("Failed to get response text.");
        assert!(response.contains("Json deserialize error"));
    }
}

/* Caller sends flowId and CJEvent value and not an AIC value
    - create a new AIC id
    - save creation time, expiration time, AIC id, flow ID, CJ event value
    - return expiration time, and AIC id
*/
#[tokio::test]
async fn aic_create_success() {
    /* SETUP */
    let (app, cj_event_value, flow_id, data) = setup_aic_test().await;
    let model = AICModel {
        db_pool: &app.db_connection(),
    };
    /* CALL */
    let r = send_post_request(&app, "/aic", data).await;
    assert_eq!(r.status(), 201);
    let response: AICResponse = r.json().await.expect("Failed to get JSON response.");
    /* TEST */
    assert_created_response(&response);
    let saved = assert_saved(
        model,
        response.aic_id,
        response.expires.unix_timestamp(),
        cj_event_value,
        flow_id,
    )
    .await;
    assert_eq!(
        (saved.created - OffsetDateTime::now_utc()).whole_minutes(),
        0
    );
}

/* Caller sends AIC id, flowId, new CJEvent value
    - keep existing AIC id
    - save new creation time, new expiration time, new flow ID, new CJ event value
    - return new expiration time, existing AIC id
*/
#[tokio::test]
async fn aic_update_with_existing_aic_and_new_flow_and_cjid() {
    /* SETUP */
    let (app, cj_event_value_orig, flow_id_orig, _data) = setup_aic_test().await;
    let model = AICModel {
        db_pool: &app.db_connection(),
    };
    let aic_orig = model
        .create(&cj_event_value_orig, &flow_id_orig)
        .await
        .expect("Failed to create test object.");
    // Make sure time has passed so timestamps are different
    std::thread::sleep(std::time::Duration::from_secs(1));
    let path = format!("/aic/{}", aic_orig.id);

    let cj_event_value_new = format!("{}{}", cj_event_value_orig, "extra");
    let flow_id_new = format!("{}{}", flow_id_orig, "extra");
    let update_data = json!({
        "cj_id": cj_event_value_new,
        "flow_id": flow_id_new,
    });

    /* CALL */
    let r = send_put_request(&app, &path, update_data).await;
    assert_eq!(r.status(), 201);
    let response: AICResponse = r.json().await.expect("Failed to get JSON response.");

    /* TEST */
    assert_eq!(aic_orig.id, response.aic_id);
    // New expires time should be later than the original
    assert!(response.expires > aic_orig.expires);
    assert_saved(
        model,
        response.aic_id,
        response.expires.unix_timestamp(),
        cj_event_value_new,
        flow_id_new,
    )
    .await;
}

/* Caller sends AIC id, flowId, existing CJEvent value
    - keep existing AIC id, creation time, expiration time, cjevent value
    - save new flow ID
    - return existing expiration time, existing AIC id
*/
#[tokio::test]
async fn aic_update_when_aic_and_cjevent_exists() {
    /* SETUP */
    let (app, cj_event_value_orig, flow_id_orig, _data) = setup_aic_test().await;
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
    let r = send_put_request(&app, &path, update_data).await;
    assert_eq!(r.status(), 201);
    let response: AICResponse = r.json().await.expect("Failed to get JSON response.");

    /* TEST */
    assert_eq!(aic_orig.id, response.aic_id);
    assert_eq!(
        response.expires.unix_timestamp(),
        aic_orig.expires.unix_timestamp()
    );
    assert_saved(
        model,
        response.aic_id,
        response.expires.unix_timestamp(),
        cj_event_value_orig,
        flow_id_new,
    )
    .await;
}

/* Caller sends flowId, CJEvent value, and AIC value but AIC doesn't exist in our DB
    - create a new AIC id
    - save creation time, expiration time, AIC id, flow ID, CJ event value
    - return expiration time, and AIC id
*/
#[tokio::test]
async fn aic_update_when_no_aic_exists() {
    /* SETUP */
    let (app, cj_event_value, flow_id, data) = setup_aic_test().await;
    let model = AICModel {
        db_pool: &app.db_connection(),
    };

    /* CALL */
    let path = format!("/aic/{}", Uuid::new_v4());
    let r = send_put_request(&app, &path, data).await;
    assert_eq!(r.status(), 201);
    let response: AICResponse = r.json().await.expect("Failed to get JSON response.");

    /* TEST */
    assert_created_response(&response);
    let saved = assert_saved(
        model,
        response.aic_id,
        response.expires.unix_timestamp(),
        cj_event_value,
        flow_id,
    )
    .await;
    assert_eq!(
        (saved.created - OffsetDateTime::now_utc()).whole_minutes(),
        0
    );
}

///// HELPERS

async fn setup_aic_test() -> (TestApp, String, String, serde_json::Value) {
    let app = spawn_app().await;
    let cj_event_value = random_ascii_string();
    let flow_id = random_ascii_string();
    let data = json!({
        "flow_id": flow_id,
        "cj_id": cj_event_value,
    });
    (app, cj_event_value, flow_id, data)
}

fn assert_created_response(response: &AICResponse) {
    // Should be UUID v4 aka Version::Random
    assert_eq!(Some(Version::Random), response.aic_id.get_version());
    /*
    Expires date is 30 days from today
    (because we created the expires a few nano seconds ago, this is a minute under 30 days)
    */
    assert_eq!(
        (response.expires - OffsetDateTime::now_utc()).whole_minutes(),
        30 * 24 * 60 - 1
    );
}

async fn assert_saved(
    model: AICModel<'_>,
    id: Uuid,
    expires_timestamp: i64,
    cj_event_value: String,
    flow_id: String,
) -> AIC {
    let saved = model.fetch_one().await.expect("Failed to get DB response.");
    assert_eq!(saved.id, id);
    assert_eq!(saved.expires.unix_timestamp(), expires_timestamp);
    assert_eq!(saved.cj_event_value, cj_event_value);
    assert_eq!(saved.flow_id, flow_id);
    saved
}
