use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

pub async fn index() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello world!"))
}

pub async fn heartbeat() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("OK"))
}

/* #[derive(Debug)]
pub struct AIC {
    id: Uuid,
    cj_event_value: String,
    flow_id: String,
} */

#[derive(Serialize, Deserialize)]
pub struct AICResponse {
    pub aic_id: Uuid,
    #[serde(with = "time::serde::rfc2822")]
    pub expires: OffsetDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct AICRequest {
    pub flow_id: String,
    pub cj_id: String,
}

pub async fn create(aic_id: Uuid, db_pool: &PgPool) -> Uuid {
    sqlx::query_as!(
        AIC,
        "INSERT INTO aic (id, cj_event_value, flow_id) VALUES ($1, $2, $3)",
        sqlx::types::Uuid::parse_str(&aic_id.to_string()).unwrap(),
        "cj_event_value",
        "flow_id"
    )
    .execute(db_pool)
    .await
    .expect("errorrring ");

    aic_id
}
pub async fn aic_create(_data: web::Json<AICRequest>, pool: web::Data<PgPool>) -> HttpResponse {
    let aic_id = Uuid::new_v4();
    let created = OffsetDateTime::now_utc();
    let expires = created.checked_add(Duration::days(30)).unwrap();

    let id = create(aic_id, pool.as_ref()).await;

    let aic_response = AICResponse {
        aic_id: id,
        expires,
    };
    HttpResponse::Created().json(aic_response)
}

pub async fn aic_update(req: HttpRequest, _data: web::Json<AICRequest>) -> impl Responder {
    let aic_id: String = req.match_info().load().unwrap();
    let aic_response = AICResponse {
        expires: OffsetDateTime::now_utc(),
        aic_id: Uuid::parse_str(&aic_id).unwrap(),
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
    fn go_here() {
        //assert!(1 == 0);
    }
}
