use std::env;
use std::fs::File;
use std::io::Read;

use lib::bigquery::client::{AccessTokenFromEnv, BQClient};
use lib::jobs::check_subscriptions::fetch_and_process_new_subscriptions;
use lib::models::aic::AICModel;
use lib::models::status_history::{Status, UpdateStatus};
use lib::models::subscriptions::{PartialSubscription, Subscription, SubscriptionModel};
use lib::settings::get_settings;
use lib::telemetry::StatsD;
use pretty_assertions::assert_eq;

use serde_json::Value;
use serial_test::serial;
use time::{date, time};
use uuid::Version;
use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

use crate::models::aic::make_fake_aic;
use crate::utils::get_test_db_pool;

fn fixture_bigquery_response() -> Value {
    let mut file = File::open("tests/fixtures/check_subscriptions_bigquery_response.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    serde_json::from_str(&data).expect("JSON was not well-formatted")
}

#[tokio::test]
#[serial]
async fn check_subscriptions() {
    // SETUP
    let settings = get_settings();
    let mock_statsd = StatsD::new(&settings);
    let db_pool = get_test_db_pool().await;
    let sub_model = SubscriptionModel { db_pool: &db_pool };
    let aic_model = AICModel { db_pool: &db_pool };

    // Happy path (there is another sub with this flow id to test that the dupe doesn't override)
    let sub_happy_flow_id = "531c7ddd31d17cbb608dcf9c8f40be89fe957c951cb5a2acd7052e6765efafcb";
    // null / empty values (no sub)
    let sub_null_flow_id = "nulls";
    let sub_empty_string_flow_id = "empty strings";
    // Archived aic (shold still make a sub)
    let sub_archived_flow_id = "1a33a74efc6b850f51b832ef5b6290e5f4d28a1dbcffef79b485e188a867c362";
    // flow id isn't in aic table (no sub)
    let sub_no_aic_entry_flow_id = "not-in-the-aic_tables";
    // Happy path 2 at the end of all the other tests to ensure we're continuing correctly
    let sub_happy_2_flow_id = "6d8c011f70525c1d04aaa9813f93a3cdfc7316b95cdc172c48b1d6b7a522d338";

    let mut aic_1 = make_fake_aic();
    aic_1.flow_id = sub_happy_flow_id.to_string();
    let mut aic_2 = make_fake_aic();
    aic_2.flow_id = sub_null_flow_id.to_string();
    let mut aic_3 = make_fake_aic();
    aic_3.flow_id = sub_empty_string_flow_id.to_string();
    let mut aic_4 = make_fake_aic();
    aic_4.flow_id = sub_happy_2_flow_id.to_string();

    for aic in [&aic_1, &aic_2, &aic_3, &aic_4] {
        aic_model
            .create_from_aic(aic)
            .await
            .expect("Could not create AIC");
    }
    let mut pre_archived = make_fake_aic();
    pre_archived.flow_id = sub_archived_flow_id.to_string();
    aic_model
        .create_archive_from_aic(&pre_archived)
        .await
        .expect("could not create AIC archive");

    // Setup fake bigquery with results to return
    env::set_var("BQ_ACCESS_TOKEN", "a token");
    let mock_bq = MockServer::start().await;
    let bq = BQClient::new("a project", AccessTokenFromEnv {}, Some(&mock_bq.uri())).await;
    let response = ResponseTemplate::new(200).set_body_json(fixture_bigquery_response());
    Mock::given(any())
        .respond_with(response)
        .mount(&mock_bq)
        .await;

    // GO
    fetch_and_process_new_subscriptions(&bq, &db_pool, &mock_statsd).await;

    // ASSERT
    let sub_1 = sub_model
        .fetch_one_by_flow_id(sub_happy_flow_id)
        .await
        .expect("Failed to get sub 1");
    let sub_2 = sub_model
        .fetch_one_by_flow_id(sub_archived_flow_id)
        .await
        .expect("Failed to get sub 2");
    let sub_3 = sub_model
        .fetch_one_by_flow_id(sub_happy_2_flow_id)
        .await
        .expect("Failed to get sub 3");

    for sub in [&sub_1, &sub_2, &sub_3] {
        // Expect all aics to be in archive now
        match aic_model.fetch_one_by_flow_id(&sub.flow_id).await {
            Err(sqlx::Error::RowNotFound) => {}
            _ => {
                panic!("This should not have happened. aic entry for flow_id {} should have been moved out of aic entry table.", &sub.flow_id);
            }
        };
        assert!(aic_model
            .fetch_one_by_flow_id_from_archive(&sub.flow_id)
            .await
            .is_ok());
        // Test that subs have a uuid as "id" (this is used for oid for cj reporting)
        assert_eq!(Some(Version::Random), sub.id.get_version());
    }
    assert_eq!(
        sub_1,
        Subscription::new(PartialSubscription {
            id: sub_1.id, // We can't know this ahead of time
            flow_id: sub_happy_flow_id.to_string(),
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
            coupons: None,
            aic_id: Some(aic_1.id),
            aic_expires: Some(aic_1.expires),
            cj_event_value: Some(aic_1.cj_event_value.to_string()),
        })
    );
    let sub_1_status_history = sub_1.get_status_history().unwrap();
    assert_eq!(sub_1_status_history.entries[0].status, Status::NotReported);
    assert_eq!(
        sub_2,
        Subscription::new(PartialSubscription {
            id: sub_2.id, // We can't know this ahead of time
            flow_id: sub_archived_flow_id.to_string(),
            subscription_id: "sub_1Ke0R3Kb9q6".to_string(),
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
            country: Some(
                "THIS IS AN ENTRY WHOSE AIC IS ALREADY IN THE ARCHIVE TABLE. IT SHOULD SUCCEED"
                    .to_string()
            ),
            coupons: None,
            aic_id: Some(pre_archived.id),
            aic_expires: Some(pre_archived.expires),
            cj_event_value: Some(pre_archived.cj_event_value),
        })
    );
    assert_eq!(
        // Sub three is the last one so all the failure cases in the test fixture should have been handled if 2 is also created.
        // TODO - LOGGING - when we add logging we could test for those logs to have been created
        sub_3,
        Subscription::new(PartialSubscription {
            id: sub_3.id, // We can't know this ahead of time
            flow_id: sub_happy_2_flow_id.to_string(),
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
            coupons: None,
            aic_id: Some(aic_4.id),
            aic_expires: Some(aic_4.expires),
            cj_event_value: Some(aic_4.cj_event_value),
        })
    );
    // Expect to NOT have certain entries from the test fixtures
    for bad_flow_id in [
        sub_null_flow_id,
        sub_empty_string_flow_id,
        sub_no_aic_entry_flow_id,
    ] {
        match sub_model.fetch_one_by_flow_id(bad_flow_id).await {
            Err(sqlx::Error::RowNotFound) => {}
            _ => {
                panic!(
                    "This should not have happened. {} should not have been saved.",
                    bad_flow_id
                );
            }
        }
    }

    // CLEAN UP
    env::remove_var("BQ_ACCESS_TOKEN");
}
