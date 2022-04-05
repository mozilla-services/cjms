use sqlx::PgPool;

use crate::models::aic::AICModel;

pub async fn archive_expired_aics(db_pool: &PgPool) {
    let aic_model = AICModel { db_pool };
    // Intentional expect. Cannot continue without
    let expired = aic_model
        .fetch_expired()
        .await
        .expect("Could not get expired AICs");
    for aic in expired {
        match aic_model.archive_aic(&aic).await {
            Ok(_) => {
                println!("Successfully archived aic: {}", &aic.id);
            }
            Err(e) => {
                println!(
                    "Could not archive aic: {}. Error: {}. Continuing...",
                    &aic.id, e
                );
                continue;
            }
        }
    }
}
