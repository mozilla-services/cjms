use sqlx::PgPool;

use crate::{
    error_and_incr, info_and_incr,
    models::aic::AICModel,
    telemetry::{LogKey, StatsD},
};

pub async fn archive_expired_aics(db_pool: &PgPool, statsd: &StatsD) {
    let aic_model = AICModel { db_pool };
    // Intentional expect. Cannot continue without
    let expired = aic_model
        .fetch_expired()
        .await
        .expect("Could not get expired AICs");
    for aic in expired {
        match aic_model.archive_aic(&aic).await {
            Ok(_) => {
                info_and_incr!(
                    statsd,
                    LogKey::CleanupAicArchive,
                    aic_id = &aic.id.to_string().as_str(),
                    "Successfully archived aic"
                );
            }
            Err(e) => {
                error_and_incr!(
                    statsd,
                    LogKey::CleanupAicArchiveFailed,
                    error = e,
                    aic_id = &aic.id.to_string().as_str(),
                    "Could not archive aic. Continuing..."
                );
                continue;
            }
        }
    }
}
