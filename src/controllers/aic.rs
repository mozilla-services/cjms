use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::aic::AICModel;

use tracing::Instrument;

#[derive(Serialize, Deserialize, Debug)]
pub struct AICResponse {
    pub aic_id: Uuid,
    #[serde(with = "time::serde::timestamp")]
    pub expires: OffsetDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct AICRequest {
    pub flow_id: String,
    pub cj_id: String,
}

pub async fn create(data: web::Json<AICRequest>, pool: web::Data<PgPool>) -> HttpResponse {
    // TODO attempt to fully instrument this function using the methods
    // documented in the book.
    // Good place to start since it is simple.
    // Do not worry about instrumentation on the model method for now. It may
    // need its own, or this may be the wrong level. But we will deal with that
    // later.

    let request_id = Uuid::new_v4();

    tracing::info!("request_id {} - Test info", request_id);
    tracing::error!("request_id {} - Test error", request_id);

    let request_span = tracing::info_span!(
        "Adding a test span with some data",
        %request_id,
        some_fake_data = "abc123",
    );
    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!(
        "Logging the query itself"
    );

    let aic = AICModel {
        db_pool: pool.as_ref(),
    };
    match aic.create(&data.cj_id, &data.flow_id).instrument(query_span).await {
        Ok(created) => {
            tracing::info!("request_id {} - Successfully created new record", request_id);
            let response = AICResponse {
                aic_id: created.id,
                expires: created.expires,
            };
            HttpResponse::Created().json(response)
        }
        Err(e) => {
            tracing::error!("request_id {} - Failed to create new record: {:?}", request_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update(
    path: web::Path<Uuid>,
    data: web::Json<AICRequest>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let aic = AICModel {
        db_pool: pool.as_ref(),
    };
    let aic_id = path.into_inner();
    let existing = aic.fetch_one_by_id(aic_id).await;
    let updated = match existing {
        Ok(existing) => {
            if existing.cj_event_value == data.cj_id {
                // Only update the flow_id
                aic.update_flow_id(aic_id, &data.flow_id).await
            } else {
                // Update both
                aic.update_flow_id_and_cj_event_value(aic_id, &data.cj_id, &data.flow_id)
                    .await
            }
        }
        Err(e) => match e {
            sqlx::Error::RowNotFound => return HttpResponse::NotFound().finish(),
            _ => return HttpResponse::InternalServerError().finish()
        },
    };

    match updated {
        Ok(updated) => {
            let response = AICResponse {
                aic_id: updated.id,
                expires: updated.expires,
            };
            HttpResponse::Created().json(response)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
