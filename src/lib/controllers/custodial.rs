use crate::version::{read_version, VERSION_FILE};
use actix_web::{Error, HttpResponse};

/*
 * Custodial Helpers
 * -----------------
 * Any small helpers that are for general maintenance purposes
 */

#[tracing::instrument(name = "request-index")]
pub async fn index() -> Result<HttpResponse, Error> {
    tracing::info!(r#type = "request-index-success");
    Ok(HttpResponse::Ok().body("Hello world!"))
}

pub async fn error_log() -> Result<HttpResponse, Error> {
    tracing::error!(r#type = "request-error-log-test", "Test error log report");
    Ok(HttpResponse::Ok().body("Error log test"))
}

pub async fn error_panic() -> Result<HttpResponse, Error> {
    panic!("This is fine. :fire:");
}

pub async fn error_capture() -> Result<HttpResponse, Error> {
    let err = "NaN".parse::<usize>().unwrap_err();
    sentry::capture_error(&err);

    Ok(HttpResponse::Ok().body("Error capture test"))
}

pub async fn heartbeat() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("OK"))
}

pub async fn version() -> Result<HttpResponse, Error> {
    let version_data = read_version(VERSION_FILE);
    Ok(HttpResponse::Ok().json(version_data))
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
