use crate::{
    error, info,
    telemetry::{LogKey, StatsD},
    version::{read_version, VERSION_FILE},
};
use actix_web::{web, Error, HttpResponse};
use std::time::Duration;
use time::OffsetDateTime;

use std::thread;

/*
 * Custodial Helpers
 * -----------------
 * Any small helpers that are for general maintenance purposes
 */

#[tracing::instrument(name = "request-index")]
pub async fn index() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello world!"))
}

pub async fn heartbeat() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("OK"))
}

pub async fn version() -> Result<HttpResponse, Error> {
    let version_data = read_version(VERSION_FILE);
    Ok(HttpResponse::Ok().json(version_data))
}

// Debug endpoints

// TODO make sure info_and_incr and error_and_incr are used in one of these
// endpoints
pub async fn log() -> Result<HttpResponse, Error> {
    info!(LogKey::Test, "Trace test with message");
    info!(
        LogKey::Test,
        key = "value",
        "Trace test with keyword arguments"
    );
    info!(
        LogKey::Test,
        "Trace test with format string: {}", "Hello world"
    );

    let err = "NaN".parse::<usize>().unwrap_err();
    error!(LogKey::Test, error = err, "Trace test error",);

    Ok(HttpResponse::Ok().body("Log test"))
}

pub async fn metrics(statsd: web::Data<StatsD>) -> Result<HttpResponse, Error> {
    statsd.incr(&LogKey::TestIncr);
    statsd.gauge(&LogKey::TestGauge, 5);

    let start = OffsetDateTime::now_utc();
    let hundred_millis = Duration::from_millis(100);
    thread::sleep(hundred_millis);

    statsd.time(&LogKey::TestTime, OffsetDateTime::now_utc() - start);
    Ok(HttpResponse::Ok().body("OK"))
}

pub async fn error_panic() -> Result<HttpResponse, Error> {
    panic!("This is fine. :fire:");
}

#[cfg(test)]
mod test_controllers_custodial {
    use super::*;

    use actix_web::body::to_bytes;
    use serde_json::from_slice;
    use serial_test::serial;

    use std::fs;

    use crate::version::VersionInfo;

    #[tokio::test]
    #[serial]
    async fn version_success() {
        fs::write(
            VERSION_FILE,
            "commit: a1b2c3\nsource: a source\nversion: the version",
        )
        .expect("Failed to write test file.");
        let response = version().await.expect("Failed to call version().");
        let body = response.into_body();
        let body_data: VersionInfo =
            from_slice(&to_bytes(body).await.expect("Failed to get body."))
                .expect("Failed to deserialize");
        assert_eq!(body_data.commit, "a1b2c3");
        assert_eq!(body_data.source, "a source");
        assert_eq!(body_data.version, "the version");
        fs::remove_file(VERSION_FILE).ok();
    }
}
