use lib::{appconfig::CJ, telemetry::TraceType};
use serial_test::serial;
use std::env;

#[tokio::test]
#[serial]
async fn test_creating_and_shutting_down_a_cj_object() {
    // Would like to:
    // - mock init tracing
    // - capture statsd calls
    env::set_var("BQ_ACCESS_TOKEN", "a token");
    let cj = CJ::new(TraceType::Test).await;
    assert!(!cj.db_pool.is_closed());
    cj.shutdown().await.expect("Failed to complete shutdown");
    assert!(cj.db_pool.is_closed());
    env::remove_var("BQ_ACCESS_TOKEN");
}
