use lib::jobs::cleanup::archive_expired_aics;
use lib::models::aic::AICModel;
use time::{Duration, OffsetDateTime};

use crate::{models::aic::make_fake_aic, utils::get_test_db_pool};

#[tokio::test]
async fn test_archive_expired_aics() {
    // SETUP
    let db_pool = get_test_db_pool().await;
    let aic_model = AICModel { db_pool: &db_pool };

    let now = OffsetDateTime::now_utc();
    // Should be archived
    let mut aic_1 = make_fake_aic();
    aic_1.expires = now - Duration::seconds(5);
    // Should not be archived
    let mut aic_2 = make_fake_aic();
    aic_2.expires = now + Duration::seconds(5);
    // Should be archived
    let mut aic_3 = make_fake_aic();
    aic_3.expires = now - Duration::seconds(5);
    for aic in [&aic_1, &aic_2, &aic_3] {
        aic_model
            .create_from_aic(aic)
            .await
            .expect("Could not create AIC");
    }

    archive_expired_aics(&db_pool).await;

    assert!(aic_model.fetch_one_by_id_from_archive(&aic_1.id).await.is_ok());
    assert!(aic_model.fetch_one_by_id_from_archive(&aic_2.id).await.is_err());
    assert!(aic_model.fetch_one_by_id_from_archive(&aic_3.id).await.is_ok());

    assert!(aic_model.fetch_one_by_id(&aic_1.id).await.is_err());
    assert!(aic_model.fetch_one_by_id(&aic_2.id).await.is_ok());
    assert!(aic_model.fetch_one_by_id(&aic_3.id).await.is_err());
}
