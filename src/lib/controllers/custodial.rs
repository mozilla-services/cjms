use actix_web::{Error, HttpResponse};
// use config::{Config, File, FileFormat};
// use serde::{Deserialize, Serialize};
use crate::version::{read_version, VERSION_FILE};

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

// TODO wrap in a macro to prevent prod compile?
pub async fn error_log() -> Result<HttpResponse, Error> {
    tracing::error!(r#type = "request-error-log-test", "Test error log report");
    Ok(HttpResponse::Ok().body("Error log test"))
}

pub async fn error_panic() -> Result<HttpResponse, Error> {
    panic!("This is fine. :fire:");
}

pub async fn heartbeat() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("OK"))
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct VersionInfo {
//     pub commit: String,
//     pub source: String,
//     pub version: String,
// }

// pub const VERSION_FILE: &str = "version.yaml";

pub async fn version() -> Result<HttpResponse, Error> {
    // let builder = Config::builder().add_source(File::new(VERSION_FILE, FileFormat::Yaml));
    // let config = builder.build().expect("Config couldn't be built.");
    // let response = match config.try_deserialize::<VersionInfo>() {
    //     Ok(result) => result,
    //     Err(e) => panic!("Config didn't match serialization. {:?}", e),
    // };
    let version_data = read_version(VERSION_FILE);
    Ok(HttpResponse::Ok().json(version_data))
}

// #[cfg(test)]
// mod test_controllers_custodial {
//     use super::*;

//     use actix_web::body::to_bytes;
//     use serde_json::from_slice;
//     use serial_test::serial;

//     use std::fs;

//     #[tokio::test]
//     #[serial]
//     #[should_panic(expected = "Config couldn't be built.")]
//     async fn missing_file_panics() {
//         fs::remove_file(VERSION_FILE).ok();
//         version().await.expect("Failed to call version().");
//     }

//     #[tokio::test]
//     #[serial]
//     #[should_panic(expected = "Config didn't match serialization.")]
//     async fn missing_values_panics() {
//         fs::write(
//             VERSION_FILE,
//             r#"
// commit: 123456
//         "#,
//         )
//         .expect("Failed to write test file.");
//         version().await.expect("Failed to call version().");
//         fs::remove_file(VERSION_FILE).ok();
//     }

//         #[tokio::test]
//         #[serial]
//         async fn returns_values_in_file() {
//             fs::write(
//                 VERSION_FILE,
//                 r#"
//     commit: 123456
//     source: a source
//     version: the version
//             "#,
//             )
//             .expect("Failed to write test file.");
//             let response = version().await.expect("Failed to call version().");
//             let body = response.into_body();
//             let body_data: VersionInfo =
//                 from_slice(&to_bytes(body).await.expect("Failed to get body."))
//                     .expect("Failed to deserialize");
//             assert_eq!(body_data.commit, "123456");
//             assert_eq!(body_data.source, "a source");
//             assert_eq!(body_data.version, "the version");
//             fs::remove_file(VERSION_FILE).ok();
//         }
// }
