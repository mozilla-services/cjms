use actix_web::{web, HttpResponse};
use sqlx::PgPool;

pub async fn list(_pool: web::Data<PgPool>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub async fn detail(path: web::Path<String>, _pool: web::Data<PgPool>) -> HttpResponse {
    println!("The path is: {}", path);
    HttpResponse::Ok().finish()
}
