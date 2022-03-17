use crate::utils::{get_db_pool, random_ascii_string};
use lib::models::aic::{AICModel, AIC};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

#[tokio::test]
async fn test_aic_model_fetch_one_by_ids() {
    let db_pool = get_db_pool().await;
    let model = AICModel { db_pool: &db_pool };
    let created = model
        .create(&random_ascii_string(), &random_ascii_string())
        .await
        .expect("Failed to create test object.");
    // id
    let result = model
        .fetch_one_by_id(&created.id)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(result, created);
    // flow id
    let result = model
        .fetch_one_by_flow_id(&created.flow_id)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(result, created);
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
    // id
    let result = model.fetch_one_by_id(&bad_id).await;
    match result {
        Err(sqlx::Error::RowNotFound) => {
            println!("Success");
        }
        _ => {
            panic!("This should not have happened.");
        }
    };
    // flow id
    let result = model.fetch_one_by_flow_id("bad_id").await;
    match result {
        Err(sqlx::Error::RowNotFound) => {
            println!("Success");
        }
        _ => {
            panic!("This should not have happened.");
        }
    };
}

#[tokio::test]
async fn test_aic_model_create_by_aic() {
    let db_pool = get_db_pool().await;
    let model = AICModel { db_pool: &db_pool };
    let aic = AIC {
        id: Uuid::new_v4(),
        flow_id: random_ascii_string(),
        cj_event_value: random_ascii_string(),
        created: OffsetDateTime::now_utc(),
        expires: OffsetDateTime::now_utc() + Duration::days(10),
    };
    model
        .create_from_aic(&aic)
        .await
        .expect("Failed to create test object.");
    let result = model
        .fetch_one_by_id(&aic.id)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(result, aic);
}
