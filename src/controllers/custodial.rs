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

pub async fn version() -> Result<HttpResponse, Error> {
    let response = VersionInfo {
        commit: String::new(),
        source: String::from("https://github.com/mozilla-services/cjms"),
        version: String::new(),
    };
    Ok(HttpResponse::Ok().json(response))
}
