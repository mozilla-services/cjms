use std::env;
use std::fs::File;
use std::io::Read;

use lib::bigquery::client::{AccessTokenFromEnv, BQClient};
use lib::jobs::check_refunds::fetch_and_process_refunds;
use lib::models::refunds::{PartialRefund, Refund, RefundModel};
use lib::models::subscriptions::SubscriptionModel;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serial_test::serial;
use time::{date, time};
use uuid::Version;
use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

use crate::models::refunds::make_fake_refund;
use crate::models::subscriptions::make_fake_sub;
use crate::utils::get_test_db_pool;

fn fixture_bigquery_response() -> Value {
    let mut file = File::open("tests/fixtures/check_refunds_bigquery_response.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    serde_json::from_str(&data).expect("JSON was not well-formatted")
}

#[tokio::test]
#[serial]
async fn check_refunds() {
    // SETUP
    let db_pool = get_test_db_pool().await;
    let sub_model = SubscriptionModel { db_pool: &db_pool };
    let refund_model = RefundModel { db_pool: &db_pool };

    // Happy path
    let refund_1_refund_id = "re_3KftCmKb9q6OnNsL0oIyzN1U";
    let refund_1_subscription_id = "sub_1KftCmKb9q6OnNsLJWrLnxyl";
    // Refund 2 tests changing state from pending to failed
    let refund_2_refund_id = "re_4mKb9q6OnNsL0oasdf39480";
    let refund_2_subscription_id = "this_one_is_already_in_refunds_table";
    // Should not be in the refunds table as invalid subscription ID
    let refund_3_refund_id = "re_3KftCmKb9q6OnNsL0oIyzN1U_1";
    // Should be in the refunds table, handle optional fields
    let refund_4_refund_id = "re_optional_fields";
    let refund_4_subscription_id = "sub_optional_fields";
    // Missing data from big query
    let refund_5_refund_id = "re_bad_data_missing_timestamp";
    let refund_5_subscription_id = "sub_bad_data";

    let mut sub_1 = make_fake_sub();
    sub_1.subscription_id = refund_1_subscription_id.to_string();
    let mut sub_2 = make_fake_sub();
    sub_2.subscription_id = refund_2_subscription_id.to_string();
    let mut sub_3 = make_fake_sub();
    sub_3.subscription_id = refund_4_subscription_id.to_string();
    let mut sub_4 = make_fake_sub();
    sub_4.subscription_id = refund_5_subscription_id.to_string();
    for sub in [&sub_1, &sub_2, &sub_3, &sub_4] {
        sub_model
            .create_from_sub(sub)
            .await
            .expect("Failed to create sub.");
    }

    let mut refund_2 = make_fake_refund();
    refund_2.refund_id = refund_2_refund_id.to_string();
    refund_2.refund_status = Some("pending".to_string());
    refund_2.subscription_id = refund_2_subscription_id.to_string();
    refund_model
        .create_from_refund(&refund_2)
        .await
        .expect("Failed to create refund 2.");

    // Setup fake bigquery with results to return
    env::set_var("BQ_ACCESS_TOKEN", "a token");
    let mock_bq = MockServer::start().await;
    let bq = BQClient::new("a project", AccessTokenFromEnv {}, Some(&mock_bq.uri())).await;
    let response = ResponseTemplate::new(200).set_body_json(fixture_bigquery_response());
    Mock::given(any())
        .respond_with(response)
        .expect(1)
        .mount(&mock_bq)
        .await;

    // GO
    fetch_and_process_refunds(bq, &db_pool).await;

    // Expect missing refunds
    for refund_id in [refund_3_refund_id, refund_5_refund_id] {
        match refund_model.fetch_one_by_refund_id(refund_id).await {
            Err(sqlx::Error::RowNotFound) => {}
            _ => {
                panic!(
                    "This should not have happened. {} should not have been saved.",
                    refund_id
                );
            }
        }
    }

    // Expectations for remaining
    let refund_1 = refund_model
        .fetch_one_by_refund_id(refund_1_refund_id)
        .await
        .expect("Failed to get refund 1");
    let refund_2 = refund_model
        .fetch_one_by_refund_id(refund_2_refund_id)
        .await
        .expect("Failed to get refund 2");
    let refund_4 = refund_model
        .fetch_one_by_refund_id(refund_4_refund_id)
        .await
        .expect("Failed to get refund 4");

    for r in &[&refund_1, &refund_2] {
        // Test that subs have a uuid as "id" (this is used for oid for cj reporting)
        assert_eq!(Some(Version::Random), r.id.get_version());
    }

    // This implicitly tests that they are marked as NotReported as "new"
    // puts NotReported status on refunds.
    assert_eq!(
        refund_1,
        Refund::new(PartialRefund {
            id: refund_1.id,
            refund_id: refund_1_refund_id.to_string(),
            subscription_id: refund_1_subscription_id.to_string(),
            refund_created: date!(2022 - 03 - 21)
                .with_time(time!(22:14:50))
                .assume_utc(),
            refund_amount: 5988,
            refund_status: Some("pending".to_string()),
            refund_reason: Some("requested_by_customer".to_string())
        })
    );
    assert_eq!(
        refund_2,
        Refund::new(PartialRefund {
            id: refund_2.id,
            refund_id: refund_2_refund_id.to_string(),
            subscription_id: refund_2.subscription_id.to_string(),
            refund_created: date!(2022 - 03 - 21)
                .with_time(time!(22:14:50))
                .assume_utc(),
            refund_amount: 5988,
            refund_status: Some("failed".to_string()),
            refund_reason: Some("fraudulent".to_string())
        })
    );
    assert_eq!(
        refund_4,
        Refund::new(PartialRefund {
            id: refund_4.id,
            refund_id: refund_4_refund_id.to_string(),
            subscription_id: refund_4_subscription_id.to_string(),
            refund_created: date!(2022 - 03 - 21)
                .with_time(time!(22:14:50))
                .assume_utc(),
            refund_amount: 5988,
            refund_status: None,
            refund_reason: None,
        })
    );

    // CLEAN UP
    env::remove_var("BQ_ACCESS_TOKEN");
}
