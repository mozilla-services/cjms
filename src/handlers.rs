use actix_web::{web, Error, HttpResponse};
use diesel::prelude::*;

use crate::db::{DbPool, DbError};
use crate::models;

pub async fn index() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello world!"))
}

pub async fn heartbeat() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("OK"))
}

// TODO this may be better off in a different file
pub fn insert_new_cj_event(
    conn: &PgConnection
) -> Result<models::NewCjEvent, DbError> {
    use crate::schema::cj_events::dsl::*;

    let new_cj_event = models::NewCjEvent {
        flow_id: String::from("abc123"),
        cj_id: String::from("xyz890"),
    };

    diesel::insert_into(cj_events).values(&new_cj_event).execute(conn)?;
    Ok(new_cj_event)
}

pub async fn test_db_insert(
    pool: web::Data<DbPool>
) -> Result<HttpResponse, Error> {
    let _new_cj_event = web::block(move || {
        let conn = pool.get()?;
        insert_new_cj_event(&conn)
    })
    .await?;
    // TODO not sure why this doesn't work ...
    // .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("Hello world!"))
}
