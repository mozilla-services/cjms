use crate::utils::{get_test_db_pool, random_ascii_string};
use lib::models::aic::{AICModel, AIC};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

fn make_fake_aic() -> AIC {
    AIC {
        id: Uuid::new_v4(),
        flow_id: random_ascii_string(),
        cj_event_value: random_ascii_string(),
        created: OffsetDateTime::now_utc(),
        expires: OffsetDateTime::now_utc() + Duration::days(10),
    }
}

#[tokio::test]
async fn test_aic_model_fetch_one_by_ids() {
    let db_pool = get_test_db_pool().await;
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
    let db_pool = get_test_db_pool().await;
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
    let db_pool = get_test_db_pool().await;
    let model = AICModel { db_pool: &db_pool };
    let aic = make_fake_aic();
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

#[tokio::test]
async fn test_aic_archive_model_fetch_one_by_ids() {
    let db_pool = get_test_db_pool().await;
    let model = AICModel { db_pool: &db_pool };
    let created = model
        .create_archive_from_aic(&make_fake_aic())
        .await
        .expect("Failed to create test object.");
    // id
    let result = model
        .fetch_one_by_id_from_archive(&created.id)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(result, created);
    // flow id
    let result = model
        .fetch_one_by_flow_id_from_archive(&created.flow_id)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(result, created);
}

#[tokio::test]
async fn test_aic_archive_model_fetch_one_by_uuid_if_not_available() {
    let db_pool = get_test_db_pool().await;
    let model = AICModel { db_pool: &db_pool };
    model
        .create_from_aic(&make_fake_aic())
        .await
        .expect("Failed to create test object.");
    let bad_id = Uuid::new_v4();
    // id
    let result = model.fetch_one_by_id_from_archive(&bad_id).await;
    match result {
        Err(sqlx::Error::RowNotFound) => {
            println!("Success");
        }
        _ => {
            panic!("This should not have happened.");
        }
    };
    // flow id
    let result = model.fetch_one_by_flow_id_from_archive("bad_id").await;
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
async fn test_aic_archive_creates_and_deletes() {
    let db_pool = get_test_db_pool().await;
    let model = AICModel { db_pool: &db_pool };
    let aic = make_fake_aic();
    model
        .create_from_aic(&aic)
        .await
        .expect("Failed to create test object.");
    model
        .archive_aic(&aic)
        .await
        .expect("Could not complete archive.");
    // aic should be in archive table and not in aic table
    model
        .fetch_one_by_id_from_archive(&aic.id)
        .await
        .expect("Could not retrieve from archive table");
    assert!(model.fetch_one_by_id(&aic.id).await.is_err());
}

#[tokio::test]
async fn test_aic_archive_does_not_delete_if_cannot_insert() {
    let db_pool = get_test_db_pool().await;
    let model = AICModel { db_pool: &db_pool };
    let aic = make_fake_aic();
    // Set a blocking archive entry to have the same flow id as the one
    // we'll attempt to archive so the transaction should fail.
    let mut blocking_archive_entry = make_fake_aic();
    blocking_archive_entry.flow_id = aic.flow_id.clone();
    model
        .create_from_aic(&aic)
        .await
        .expect("Failed to create aic.");

    model
        .create_archive_from_aic(&blocking_archive_entry)
        .await
        .expect("Failed to create aic archive.");
    match model.archive_aic(&aic).await {
        Ok(_) => panic!("Transaction did not fail as expected."),
        Err(_) => {
            // aic should still be in aic table
            model
                .fetch_one_by_id(&aic.id)
                .await
                .expect("Could not retrieve from archive table");
            // aic should not be in archive table
            assert!(model.fetch_one_by_id_from_archive(&aic.id).await.is_err());
        }
    }
}
