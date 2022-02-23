use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

pub async fn index() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello world!"))
}

pub async fn heartbeat() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("OK"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AICResponse {
    pub aic_id: String,
    pub expires: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AICRequest {
    pub flow_id: String,
    pub cj_id: String,
}

pub async fn aic_create(_data: web::Json<AICRequest>) -> impl Responder {
    let aic_response = AICResponse {
        aic_id: "123ABC".to_string(),
        expires: "Fri, 28 Nov 2014 12:00:09 +0000".to_string(),
    };
    HttpResponse::Created().json(aic_response)
}

pub async fn aic_update(req: HttpRequest, _data: web::Json<AICRequest>) -> impl Responder {
    let aic_id: String = req.match_info().load().unwrap();
    let aic_response = AICResponse {
        expires: "Fri, 28 Nov 2014 12:00:09 +0000".to_string(),
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
