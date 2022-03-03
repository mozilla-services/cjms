use actix_web::{Error, HttpResponse};
use serde::{Deserialize, Serialize};

/*
 * Custodial Helpers
 * -----------------
 * Any small helpers that are for general maintenance purposes
 */

pub async fn index() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello world!"))
}

pub async fn heartbeat() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("OK"))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionInfo {
    pub commit: String,
    pub source: String,
    pub version: String,
}

const VERSION_FILE: &str = "version.yaml";

pub async fn version() -> Result<HttpResponse, Error> {
    println!("version file is: {}", VERSION_FILE);
    let response = VersionInfo {
        commit: String::new(),
        source: String::from("https://github.com/mozilla-services/cjms"),
        version: String::new(),
    };
    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::body::to_bytes;
    use serde_json::from_slice;
    use serial_test::serial;

    use std::fs;

    #[tokio::test]
    #[serial]
    #[should_panic(expected = "Config didn't match serialization.")]
    async fn missing_values_panics() {
        fs::remove_file(VERSION_FILE).ok();
        version().await.expect("Failed to call version().");
    }

    #[tokio::test]
    #[serial]
    async fn returns_values_in_file() {
        fs::remove_file(VERSION_FILE).ok();
        fs::write(
            VERSION_FILE,
            r#"
commit: 123456
source: a source
version: the version
        "#,
        )
        .expect("Failed to write test file: {}");
        let response = version().await.expect("Failed to call version().");
        let body = response.into_body();
        let body_data: VersionInfo =
            from_slice(&to_bytes(body).await.expect("Failed to get body."))
                .expect("Failed to deserialize");
        assert_eq!(body_data.commit, "123456");
        assert_eq!(body_data.source, "a source");
        assert_eq!(body_data.version, "the version");
    }
}
