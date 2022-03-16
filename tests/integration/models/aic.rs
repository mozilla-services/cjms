use crate::utils::{get_db_pool, random_ascii_string};
use lib::models::aic::AICModel;
use uuid::Uuid;

#[tokio::test]
async fn test_aic_model_fetch_one_by_uuid() {
    let db_pool = get_db_pool().await;
    let model = AICModel { db_pool: &db_pool };
    let created = model
        .create(&random_ascii_string(), &random_ascii_string())
        .await
        .expect("Failed to create test object.");
    let result = model
        .fetch_one_by_id(created.id)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(result.id, created.id);
}

#[tokio::test]
async fn test_aic_model_fetch_one_by_uuid_if_not_available() {
    let db_pool = get_db_pool().await;
    let model = AICModel { db_pool: &db_pool };
    model
        .create(&random_ascii_string(), &random_ascii_string())
        .await
        .expect("Failed to create test object.");
    let bad_id = Uuid::new_v4();
    let result = model.fetch_one_by_id(bad_id).await;
    match result {
        Err(sqlx::Error::RowNotFound) => {
            println!("Success");
        }
        _ => {
            panic!("This should not have happened.");
        }
    };
}
