use crate::utils::{
    get_length_of_subscription_status_history_array,
    get_value_from_subscription_status_history_array, random_ascii_string,
    random_currency_or_country, random_price, spawn_app,
};
use lib::models::subscriptions::{Subscription, SubscriptionModel};
use pretty_assertions::assert_eq;
use serde_json::json;
use time::{Duration, OffsetDateTime, PrimitiveDateTime};
use uuid::Uuid;

pub fn make_fake_sub() -> Subscription {
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
        country: Some(random_currency_or_country()),
        aic_id: Some(Uuid::new_v4()),
        aic_expires: Some(OffsetDateTime::now_utc()),
        cj_event_value: Some(random_ascii_string()),
        status: Some(random_ascii_string()),
        status_history: Some(json!([])),
    }
}

pub async fn save_sub(model: &SubscriptionModel<'_>, sub: &Subscription) {
    model
        .create_from_sub(sub)
        .await
        .expect("Failed to create test object.");
}

#[tokio::test]
async fn test_subscription_model_create_from_subscription_struct() {
    let app = spawn_app().await;
    let model = SubscriptionModel {
        db_pool: &app.db_connection(),
    };
    let sub = make_fake_sub();
    save_sub(&model, &sub).await;
    let result = model
        .fetch_one_by_id(&sub.id)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(result, sub);
}

#[tokio::test]
async fn test_subscription_model_fetch_ones_by_unique_ids() {
    let app = spawn_app().await;
    let model = SubscriptionModel {
        db_pool: &app.db_connection(),
    };
    let sub = make_fake_sub();
    save_sub(&model, &sub).await;
    let result = model
        .fetch_one_by_id(&sub.id)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(sub, result);
    let result = model
        .fetch_one_by_flow_id(&sub.flow_id)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(sub, result);
}

#[tokio::test]
async fn test_subscription_model_fetch_ones_if_not_available() {
    let app = spawn_app().await;
    let model = SubscriptionModel {
        db_pool: &app.db_connection(),
    };
    let sub = make_fake_sub();
    save_sub(&model, &sub).await;
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
    let result = model.fetch_one_by_flow_id("nope").await;
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
async fn test_subscription_model_get_all_not_reported() {
    let app = spawn_app().await;
    let model = SubscriptionModel {
        db_pool: &app.db_connection(),
    };
    let mut sub_1 = make_fake_sub();
    sub_1.status = Some("not_reported".to_string());
    save_sub(&model, &sub_1).await;
    let mut sub_2 = make_fake_sub();
    sub_2.status = Some("a status".to_string());
    save_sub(&model, &sub_2).await;
    let mut sub_3 = make_fake_sub();
    sub_3.status = Some("not_reported".to_string());
    save_sub(&model, &sub_3).await;
    let all = model.fetch_all().await.unwrap();
    assert_eq!(all.len(), 3);
    let result = model
        .fetch_all_not_reported()
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(result.len(), 2);
    assert!(result.contains(&sub_1));
    assert!(result.contains(&sub_3));
}

#[tokio::test]
async fn test_subscription_model_mark_reported() {
    let app = spawn_app().await;
    let model = SubscriptionModel {
        db_pool: &app.db_connection(),
    };
    let mut sub = make_fake_sub();
    sub.status = Some("not_reported".to_string());
    save_sub(&model, &sub).await;
    assert_eq!(
        get_length_of_subscription_status_history_array(&sub.status_history.unwrap()),
        0
    );
    model
        .mark_sub_reported(&sub.id)
        .await
        .expect("Should not fail.");
    let result = model.fetch_one_by_id(&sub.id).await.unwrap();
    assert_eq!(result.status, Some("reported".to_string()));
    let result_status_history = result.status_history.unwrap();
    assert_eq!(
        get_length_of_subscription_status_history_array(&result_status_history),
        1
    );
    let status_in_status_history =
        get_value_from_subscription_status_history_array(&result_status_history, 0, "status");
    assert_eq!(status_in_status_history, "reported");
    let status_time_string =
        get_value_from_subscription_status_history_array(&result_status_history, 0, "t");
    let t_in_status_history = PrimitiveDateTime::parse(status_time_string, "%Y-%m-%d %T.%N +0")
        .unwrap()
        .assume_utc();
    // This won't pass if the test is slower than a second to process
    assert_eq!(
        t_in_status_history.unix_timestamp(),
        OffsetDateTime::now_utc().unix_timestamp()
    );
}
