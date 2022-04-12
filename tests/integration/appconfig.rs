use lib::{
    appconfig::CJ,
    telemetry::TraceType,
    version::{write_version, VersionInfo, VERSION_FILE},
};
use serial_test::serial;
use std::env;
use std::fs;

#[tokio::test]
#[serial]
async fn test_creating_and_shutting_down_a_cj_object() {
    // Would like to:
    // - mock init tracing
    // - capture statsd calls

    let version_data = VersionInfo {
        commit: "a1b2c3".to_string(),
        source: "source".to_string(),
        version: "version".to_string(),
    };
    write_version(VERSION_FILE, &version_data);
    env::set_var("BQ_ACCESS_TOKEN", "a token");

    let cj = CJ::new(TraceType::Test).await;
    assert!(!cj.db_pool.is_closed());
    cj.shutdown().await.expect("Failed to complete shutdown");
    assert!(cj.db_pool.is_closed());

    env::remove_var("BQ_ACCESS_TOKEN");
    fs::remove_file(VERSION_FILE).unwrap();
}
