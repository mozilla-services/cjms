use actix_web::{Error, HttpResponse};
use serde::{Deserialize, Serialize};


pub async fn index() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello world!"))
}

pub async fn heartbeat() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("OK"))
}

#[derive(Serialize, Deserialize)]
pub struct AICResponse {
    pub aic_id: String,
    pub expires: String,
}

pub async fn aic_create() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Created().body("OK"))
}

pub async fn aic_update() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Created().body("OK"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // AIC endpoing unit tests
    // sanitize input to post endpoint before put in db
    // expiration time based on environment variable
    #[test]
    fn tests_go_here() {
        aic_create();
    }
}