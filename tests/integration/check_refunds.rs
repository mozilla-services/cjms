use std::env;
use std::fs::File;
use std::io::Read;

use lib::bigquery::client::{AccessTokenFromEnv, BQClient};
use lib::check_refunds::fetch_and_process_refunds;
//use pretty_assertions::assert_eq;

use serde_json::Value;
use serial_test::serial;
use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

use crate::utils::get_db_pool;

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
    let db_pool = get_db_pool().await;

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
    fetch_and_process_refunds(bq, &db_pool).await;

    // CLEAN UP
    env::remove_var("BQ_ACCESS_TOKEN");
}
