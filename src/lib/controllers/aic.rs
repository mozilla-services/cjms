use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    error_and_incr, info_and_incr,
    models::aic::AICModel,
    settings::Settings,
    telemetry::{LogKey, StatsD},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct AICResponse {
    pub aic_id: Uuid,
    #[serde(with = "time::serde::timestamp")]
    pub expires: OffsetDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct AICRequest {
    pub flow_id: String,
    #[serde(default = "empty_cj_id")]
    pub cj_id: String,
}

fn empty_cj_id() -> String {
    "empty_cj_id".to_string()
}

pub async fn create(
    data: web::Json<AICRequest>,
    pool: web::Data<PgPool>,
    settings: web::Data<Settings>,
    statsd: web::Data<StatsD>,
) -> HttpResponse {
    let aic = AICModel {
        db_pool: pool.as_ref(),
    };
    match aic.create(&data.cj_id, &data.flow_id, &settings).await {
        Ok(created) => {
            info_and_incr!(
                statsd,
                LogKey::AicRecordCreate,
                aic_id = created.id.to_string().as_str(),
                flow_id = created.flow_id.as_str(),
                expired = created.expires.to_string().as_str(),
                "AIC created."
            );
            let response = AICResponse {
                aic_id: created.id,
                expires: created.expires,
            };
            HttpResponse::Created().json(response)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn update(
    path: web::Path<Uuid>,
    data: web::Json<AICRequest>,
    pool: web::Data<PgPool>,
    settings: web::Data<Settings>,
    statsd: web::Data<StatsD>,
) -> HttpResponse {
    let aic = AICModel {
        db_pool: pool.as_ref(),
    };
    let aic_id = path.into_inner();
    let existing = aic.fetch_one_by_id(&aic_id).await;
    let updated = match existing {
        Ok(existing) => {
            if existing.cj_event_value == data.cj_id || data.cj_id == empty_cj_id() {
                // Only update the flow_id
                aic.update_flow_id(aic_id, &data.flow_id).await
            } else {
                // Update both
                aic.update_flow_id_and_cj_event_value(aic_id, &data.cj_id, &data.flow_id, &settings)
                    .await
            }
        }
        Err(e) => match e {
            sqlx::Error::RowNotFound => {
                error_and_incr!(
                    statsd,
                    LogKey::AicRecordUpdateFailedNotFound,
                    aic_id = &aic_id.to_string().as_str(),
                    "AIC could not be found."
                );
                return HttpResponse::NotFound().finish();
            }
            _ => {
                error_and_incr!(
                    statsd,
                    LogKey::AicRecordUpdateFailed,
                    error = e,
                    aic_id = &aic_id.to_string().as_str(),
                    "AIC update failed."
                );
                return HttpResponse::InternalServerError().finish();
            }
        },
    };

    match updated {
        Ok(updated) => {
            if updated.cj_event_value == data.cj_id || data.cj_id == empty_cj_id() {
                info_and_incr!(
                    statsd,
                    LogKey::AicRecordUpdate,
                    aic_id = updated.id.to_string().as_str(),
                    flow_id = updated.flow_id.as_str(),
                    expires = updated.expires.to_string().as_str(),
                    "AIC updated."
                );
            } else {
                info_and_incr!(
                    statsd,
                    LogKey::AicRecordUpdate,
                    aic_id = updated.id.to_string().as_str(),
                    flow_id = updated.flow_id.as_str(),
                    expires = updated.expires.to_string().as_str(),
                    cj_event_value = updated.cj_event_value.as_str(),
                    "AIC updated."
                );
            }

            let response = AICResponse {
                aic_id: updated.id,
                expires: updated.expires,
            };
            HttpResponse::Created().json(response)
        }
        Err(e) => {
            error_and_incr!(
                statsd,
                LogKey::AicRecordUpdateFailed,
                error = e,
                aic_id = &aic_id.to_string().as_str(),
                "AIC update failed."
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
