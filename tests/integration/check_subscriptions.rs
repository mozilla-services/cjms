use std::env;
use std::fs::File;
use std::io::Read;

use lib::bigquery::client::{AccessTokenFromEnv, BQClient};
use lib::check_subscriptions::fetch_and_process_new_subscriptions;
use lib::models::aic::{AICModel, AIC};
use lib::models::subscriptions::{Subscription, SubscriptionModel};
use serde::Deserialize;
use serde_json::{json, Value};
use serial_test::serial;
use time::{Format, OffsetDateTime};
use uuid::Uuid;
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

fn test_aics() -> Vec<AIC> {
    let mut file = File::open("tests/fixtures/subscriptions_aic.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let imported: Vec<AICSimple> =
        serde_json::from_str(&data).expect("JSON was not well-formatted");
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
    let mut file = File::open("tests/fixtures/bigquery_generic_response.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    serde_json::from_str(&data).expect("JSON was not well-formatted")
}

#[tokio::test]
#[serial]
async fn check_subscriptions() {
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
    let db_pool = get_db_pool().await;
    let aic_model = AICModel { db_pool: &db_pool };
    for aic in test_aics() {
        aic_model
            .create_from_aic(&aic)
            .await
            .unwrap_or_else(|_| panic!("Failed to create aic: {:?}", &aic));
    }

    // Go!
    fetch_and_process_new_subscriptions(bq).await;

    // Check results
    let sub_model = SubscriptionModel { db_pool: &db_pool };
    let sub_1_flow_id = "531c7ddd31d17cbb608dcf9c8f40be89fe957c951cb5a2acd7052e6765efafcb";
    let sub_2_flow_id = "6d8c011f70525c1d04aaa9813f93a3cdfc7316b95cdc172c48b1d6b7a522d338";

    let sub_1 = sub_model
        .fetch_one_by_flow_id(sub_1_flow_id)
        .await
        .expect("Failed to get sub 1");
    let _sub_2 = sub_model
        .fetch_one_by_flow_id(sub_2_flow_id)
        .await
        .expect("Failed to get sub 2");

    let aic_1 = aic_model.fetch_one().await.expect("Failed to fetch aic_1");
    let _aic_2 = aic_model.fetch_one().await.expect("Failed to fetch aic_2");

    assert_eq!(
        sub_1,
        Subscription {
            id: sub_1.id, // We can't know this ahead of time
            flow_id: sub_1_flow_id.to_string(),
            subscription_id: "sub_1Ke0R3Kb9q6OnNsLD1OIZsxm".to_string(),
            report_timestamp: OffsetDateTime::parse(
                "2022-03-16 20:59:53.672304 UTC",
                Format::Rfc3339
            )
            .unwrap(),
            subscription_created: OffsetDateTime::parse("2022-03-16 17:14:57 UTC", Format::Rfc3339)
                .unwrap(),
            fxa_uid: "37794607f1f1a8f9ad310d32d84e606cd8884c0d965d1036316d8ab64892b1f7".to_string(),
            quantity: 1,
            plan_id: "price_1J0owvKb9q6OnNsLExNhEDXm".to_string(),
            plan_currency: "usd".to_string(),
            plan_amount: 100,
            country: "".to_string(),
            aic_id: Some(aic_1.id),
            cj_event_value: Some(aic_1.flow_id.to_string()),
            status: "not_reported".to_string(),
            status_history: json!([{
                "status": "not_reported",
                "timestamp": ""
            }])
        }
    );

    // Expect 2 new subs to be created
    // Expect aic table to no longer have the two new subs
    // Expect aic archive to have the two new subs

    // TODO
    // - Add in a sub response with a flow id we don't have
    // - Add in a sub response with an aic id that's in the archive table

    // Clean-up
    env::remove_var("BQ_ACCESS_TOKEN");
}
