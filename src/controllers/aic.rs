use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::aic::AICModel;

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

pub async fn create(_data: web::Json<AICRequest>, pool: web::Data<PgPool>) -> HttpResponse {
    let aic = AICModel {
        db_pool: pool.as_ref(),
    };
    let created = aic.create().await;
    let response = AICResponse {
        aic_id: created.id,
        expires: created.expires,
    };
    HttpResponse::Created().json(response)
}

pub async fn update(_req: HttpRequest, _data: web::Json<AICRequest>) -> HttpResponse {
    todo!();
}
