use actix_web::{web, HttpResponse};
use sqlx::PgPool;

pub async fn today(_pool: web::Data<PgPool>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub async fn by_day(path: web::Path<String>, _pool: web::Data<PgPool>) -> HttpResponse {
    let today = path.into_inner();
    HttpResponse::Ok().body(today)
}
