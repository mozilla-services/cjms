use actix_web::{Error, HttpResponse};

pub async fn check() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Check subscriptions"))
}
