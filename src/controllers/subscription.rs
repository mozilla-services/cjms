use actix_web::{web, Error, HttpResponse};
use sqlx::PgPool;

use crate::models::subscription::SubscriptionModel;

pub async fn check_subscriptions(pool: &PgPool) {
    let subs = SubscriptionModel { db_pool: pool };
    for _ in 0..3 {
        subs.create().await.expect("Create failed :(");
    }
}

pub async fn check(pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    check_subscriptions(pool.as_ref()).await;
    Ok(HttpResponse::Ok().body("Check subscriptions"))
}
