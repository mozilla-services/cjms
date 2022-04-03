use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;

use crate::settings::Settings;

#[derive(Deserialize)]
pub struct CorrectionsPath {
    id: String,
}

pub async fn detail(
    path: web::Path<CorrectionsPath>,
    settings: web::Data<Settings>,
    _pool: web::Data<PgPool>,
) -> HttpResponse {
    if !path.id.eq(&settings.cj_signature) {
        return HttpResponse::NotFound().finish();
    }
    HttpResponse::Ok().finish()
}
