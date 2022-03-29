use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(_pool: web::Data<PgPool>) -> HttpResponse {
    HttpResponse::Forbidden().finish()
}

pub async fn detail(_path: web::Path<Uuid>, _pool: web::Data<PgPool>) -> HttpResponse {
    HttpResponse::Forbidden().finish()
}
