use std::env;
use std::fs::File;
use std::io::Read;

use lib::bigquery::client::{AccessTokenFromEnv, BQClient};
use lib::check_subscriptions::fetch_and_process_new_subscriptions;
use lib::models::aic::{AICModel, AIC};
use lib::models::subscriptions::{Subscription, SubscriptionModel};
use pretty_assertions::assert_eq;
use serde::Deserialize;
use serde_json::Value;
use serial_test::serial;
use time::{date, time, Format, OffsetDateTime};
use uuid::{Uuid, Version};
use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

use crate::utils::get_db_pool;

#[derive(Debug, Deserialize)]
pub struct AICSimple {
    pub id: Uuid,
    pub cj_event_value: String,
    pub flow_id: String,
    pub created: String,
    pub expires: String,
}

fn get_aics_from_json(path: &str) -> Vec<AIC> {
    let mut file = File::open(path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let imported: Vec<AICSimple> =
        serde_json::from_str(&data).expect("Invalid JSON.");
    let mut aics: Vec<AIC> = vec![];
    for aic in imported {
        let x = AIC {
            id: aic.id,
            cj_event_value: aic.cj_event_value,
            flow_id: aic.flow_id,
            created: OffsetDateTime::parse(aic.created, Format::Rfc3339).unwrap(),
            expires: OffsetDateTime::parse(aic.expires, Format::Rfc3339).unwrap(),
        };
        aics.push(x);
    }
    aics
}

fn fixture_bigquery_response() -> Value {
    let mut file = File::open("tests/fixtures/check_subscriptions_bigquery_response.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    serde_json::from_str(&data).expect("JSON was not well-formatted")
}

fn get_value_from_status_history_array(
    status_history: &Value,
    array_index: usize,
    field_name: &str,
) -> String {
    let array = status_history.as_array().unwrap();
    let array_entry = array[array_index].as_object().unwrap();
    let entry_value = array_entry.get(field_name).unwrap().to_string();
    entry_value
}

#[tokio::test]
#[serial]
async fn check_subscriptions() {
    // SETUP
    let db_pool = get_db_pool().await;
    let sub_model = SubscriptionModel { db_pool: &db_pool };
    let aic_model = AICModel { db_pool: &db_pool };
    // Setup fake bigquery with results to return
    env::set_var("BQ_ACCESS_TOKEN", "a token");
    let mock_bq = MockServer::start().await;
    let bq = BQClient::new("a project", AccessTokenFromEnv {}, Some(&mock_bq.uri())).await;
    let response = ResponseTemplate::new(200).set_body_json(fixture_bigquery_response());
    Mock::given(any())
        .respond_with(response)
        .mount(&mock_bq)
        .await;
    // Setup AIC entries
    for aic in get_aics_from_json("tests/fixtures/check_subscriptions_aic.json") {
        aic_model
            .create_from_aic(&aic)
            .await
            .unwrap_or_else(|_| panic!("Failed to create aic: {:?}", &aic));
    }
    for aic in get_aics_from_json("tests/fixtures/check_subscriptions_aic_archive.json") {
        aic_model
            .create_archive_from_aic(&aic)
            .await
            .unwrap_or_else(|_| panic!("Failed to create aic: {:?}", &aic));
    }

    // GO
    fetch_and_process_new_subscriptions(bq, &db_pool).await;

    // ASSERT
    let sub_1_flow_id = "531c7ddd31d17cbb608dcf9c8f40be89fe957c951cb5a2acd7052e6765efafcb";
    let sub_2_flow_id = "6d8c011f70525c1d04aaa9813f93a3cdfc7316b95cdc172c48b1d6b7a522d338";

    let sub_1 = sub_model
        .fetch_one_by_flow_id(sub_1_flow_id)
        .await
        .expect("Failed to get sub 1");
    let sub_2 = sub_model
        .fetch_one_by_flow_id(sub_2_flow_id)
        .await
        .expect("Failed to get sub 2");
    for flow_id in &[&sub_1.flow_id, &sub_2.flow_id] {
        // Expect aic table to no longer have the two new subs
        match aic_model.fetch_one_by_flow_id(flow_id).await {
            Err(sqlx::Error::RowNotFound) => {}
            _ => {
                panic!("This should not have happened. aic entry for flow_id {} should have been moved out of aic entry table.", flow_id);
            }
        }
    }
    for sub in &[&sub_1, &sub_2] {
        // Test that subs have a uuid as "id" (this is used for oid for cj reporting)
        assert_eq!(Some(Version::Random), sub.id.get_version());
    }
    // Expect aic archive to have the two new subs
    let aic_1 = aic_model
        .fetch_one_by_flow_id_from_archive(&sub_1.flow_id)
        .await
        .expect("Failed to fetch aic_1");
    let aic_2 = aic_model
        .fetch_one_by_flow_id_from_archive(&sub_2.flow_id)
        .await
        .expect("Failed to fetch aic_2");
    assert_eq!(
        sub_1,
        Subscription {
            id: sub_1.id, // We can't know this ahead of time
            flow_id: sub_1_flow_id.to_string(),
            subscription_id: "sub_1Ke0R3Kb9q6OnNsLD1OIZsxm".to_string(),
            report_timestamp: date!(2022 - 03 - 16)
                .with_time(time!(20:59:53))
                .assume_utc(),
            subscription_created: date!(2022 - 03 - 16)
                .with_time(time!(17:14:57))
                .assume_utc(),
            fxa_uid: "37794607f1f1a8f9ad310d32d84e606cd8884c0d965d1036316d8ab64892b1f7".to_string(),
            quantity: 1,
            plan_id: "price_1J0owvKb9q6OnNsLExNhEDXm".to_string(),
            plan_currency: "usd".to_string(),
            plan_amount: 100,
            country: Some("us - THIS IS SUB 1".to_string()),
            aic_id: Some(aic_1.id),
            aic_expires: Some(aic_1.expires),
            cj_event_value: Some(aic_1.cj_event_value.to_string()),
            status: Some("not_reported".to_string()),
            status_history: None, // This field isn't compared
        }
    );
    let sub_1_status_history_0_status =
        get_value_from_status_history_array(&sub_1.status_history.unwrap(), 0, "status");
    assert_eq!(&sub_1_status_history_0_status, r#""not_reported""#);
    assert_eq!(
        // Sub two is the last one so all the failure cases in the test fixture should have been handled if 2 is also created.
        // TODO - LOGGING - when we add logging we could test for those logs to have been created
        sub_2,
        Subscription {
            id: sub_2.id, // We can't know this ahead of time
            flow_id: sub_2_flow_id.to_string(),
            subscription_id: "sub_1Ke0CHKb9q6OnNsLe2fSFt2W".to_string(),
            report_timestamp: date!(2022 - 03 - 16)
                .with_time(time!(20:59:53))
                .assume_utc(),
            subscription_created: date!(2022 - 03 - 16)
                .with_time(time!(16:59:41))
                .assume_utc(),
            fxa_uid: "bc5bdcb1c00baf74c85d19413c3889d4653c9a79f5715a45389241ef6fc51ecb".to_string(),
            quantity: 1,
            plan_id: "price_1J0Y12Kb9q6OnNsL4SB2hhmp".to_string(),
            plan_currency: "usd".to_string(),
            plan_amount: 4794,
            country: None,
            aic_id: Some(aic_2.id),
            aic_expires: Some(aic_2.expires),
            cj_event_value: Some(aic_2.cj_event_value),
            status: Some("not_reported".to_string()),
            status_history: None, // This field isn't compared
        }
    );
    // Expect to NOT have certain entries from the test fixtures
    assert!(sub_model.fetch_one_by_flow_id("nulls").await.is_err());
    assert!(sub_model
        .fetch_one_by_flow_id("empty strings")
        .await
        .is_err());
    assert!(sub_model
        .fetch_one_by_flow_id("not-in-the-aic-tables")
        .await
        .is_err());

    // CLEAN UP
    env::remove_var("BQ_ACCESS_TOKEN");
}
