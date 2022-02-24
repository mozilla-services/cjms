use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

pub async fn index() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello world!"))
}

pub async fn heartbeat() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("OK"))
}

// TODO - Do I need all these traits?
#[derive(Debug, Serialize, Deserialize)]
pub struct AICResponse {
    pub aic_id: String,
    #[serde(with = "time::serde::rfc2822")]
    pub expires: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AICRequest {
    pub flow_id: String,
    pub cj_id: String,
}

pub async fn aic_create(_data: web::Json<AICRequest>) -> impl Responder {
    let aic_id = Uuid::new_v4();
    let created = OffsetDateTime::now_utc();
    let expires = created.checked_add(Duration::days(30)).unwrap();
    let aic_response = AICResponse {
        aic_id: aic_id.to_string(),
        expires,
    };
    HttpResponse::Created().json(aic_response)
}

pub async fn aic_update(req: HttpRequest, _data: web::Json<AICRequest>) -> impl Responder {
    let aic_id: String = req.match_info().load().unwrap();
    let aic_response = AICResponse {
        expires: OffsetDateTime::now_utc(),
        aic_id,
    };
    HttpResponse::Created().json(aic_response)
}

#[cfg(test)]
mod tests {
    //use super::*;

    // AIC endpoing unit tests
    // sanitize input to post endpoint before put in db
    // expiration time based on environment variable
    #[test]
    fn tests_go_here() {
        //assert!(1 == 0);
    }
}
