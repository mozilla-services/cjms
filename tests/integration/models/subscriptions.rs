use crate::utils::{random_ascii_string, random_currency_or_country, random_price, spawn_app};
use lib::models::subscriptions::{Subscription, SubscriptionModel};
use serde_json::json;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

fn make_fake_sub() -> Subscription {
    Subscription {
        id: Uuid::new_v4(),
        flow_id: random_ascii_string(),
        subscription_id: random_ascii_string(),
        report_timestamp: OffsetDateTime::now_utc(),
        subscription_created: OffsetDateTime::now_utc() - Duration::hours(35),
        fxa_uid: random_ascii_string(),
        quantity: 1,
        plan_id: random_ascii_string(),
        plan_currency: random_currency_or_country(),
        plan_amount: random_price(),
        country: random_currency_or_country(),
        aic_id: Some(Uuid::new_v4()),
        cj_event_value: Some(random_ascii_string()),
        status: random_ascii_string(),
        status_history: json!([]),
    }
}

#[tokio::test]
async fn test_subscription_model_create_from_subscription_struct() {
    let app = spawn_app().await;
    let model = SubscriptionModel {
        db_pool: &app.db_connection(),
    };
    let sub = make_fake_sub();
    model
        .create_from_sub(&sub)
        .await
        .expect("Failed to create test object.");
    let result = model
        .fetch_one_by_id(&sub.id)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(result.aic_id, sub.aic_id);
}

#[tokio::test]
async fn test_subscription_model_fetch_ones_by_unique_ids() {
    let app = spawn_app().await;
    let model = SubscriptionModel {
        db_pool: &app.db_connection(),
    };
    let sub = make_fake_sub();
    model
        .create_from_sub(&sub)
        .await
        .expect("Failed to create test object.");
    let result = model
        .fetch_one_by_id(&sub.id)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(sub.aic_id, result.aic_id);
}

#[tokio::test]
async fn test_aic_model_fetch_one_if_not_available() {
    let app = spawn_app().await;
    let model = SubscriptionModel {
        db_pool: &app.db_connection(),
    };
    let sub = make_fake_sub();
    model
        .create_from_sub(&sub)
        .await
        .expect("Failed to create test object.");
    let bad_id = Uuid::new_v4();
    let result = model.fetch_one_by_id(&bad_id).await;
    match result {
        Err(sqlx::Error::RowNotFound) => {
            println!("Success");
        }
        _ => {
            panic!("This should not have happened.");
        }
    };
}
