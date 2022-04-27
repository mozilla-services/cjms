use crate::utils::{
    get_test_db_pool, random_ascii_string, random_currency_or_country, random_price,
    random_simple_ascii_string,
};
use lib::models::{
    status_history::{Status, UpdateStatus},
    subscriptions::{PartialSubscription, Subscription, SubscriptionModel},
};
use pretty_assertions::assert_eq;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

pub fn make_fake_sub() -> Subscription {
    Subscription::new(PartialSubscription {
        id: Uuid::new_v4(),
        flow_id: random_simple_ascii_string(),
        subscription_id: random_simple_ascii_string(),
        report_timestamp: OffsetDateTime::now_utc(),
        subscription_created: OffsetDateTime::now_utc() - Duration::hours(35),
        fxa_uid: random_ascii_string(),
        quantity: 1,
        plan_id: random_simple_ascii_string(),
        plan_currency: random_currency_or_country(),
        plan_amount: random_price(),
        country: Some(random_currency_or_country()),
        aic_id: Some(Uuid::new_v4()),
        aic_expires: Some(OffsetDateTime::now_utc()),
        cj_event_value: Some(random_ascii_string()),
    })
}

pub async fn save_sub(model: &SubscriptionModel<'_>, sub: &Subscription) {
    model
        .create_from_sub(sub)
        .await
        .expect("Failed to create test object.");
}

#[tokio::test]
async fn test_subscription_model_create_from_subscription() {
    let db_pool = get_test_db_pool().await;
    let model = SubscriptionModel { db_pool: &db_pool };
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
    let db_pool = get_test_db_pool().await;
    let model = SubscriptionModel { db_pool: &db_pool };
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
    let result = model
        .fetch_one_by_subscription_id(&sub.subscription_id)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(sub, result);
}

#[tokio::test]
async fn test_subscription_model_fetch_ones_if_not_available() {
    let db_pool = get_test_db_pool().await;
    let model = SubscriptionModel { db_pool: &db_pool };
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
    let result = model.fetch_one_by_subscription_id("nope").await;
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
async fn test_subscription_model_get_all_by_status() {
    let db_pool = get_test_db_pool().await;
    let model = SubscriptionModel { db_pool: &db_pool };
    let sub_1 = make_fake_sub();
    save_sub(&model, &sub_1).await;

    let mut sub_2 = make_fake_sub();
    sub_2.update_status(Status::Reported);
    save_sub(&model, &sub_2).await;

    let sub_3 = make_fake_sub();
    save_sub(&model, &sub_3).await;

    let all = model.fetch_all().await.unwrap();
    assert_eq!(all.len(), 3);

    let not_reported = model
        .fetch_all_by_status(Status::NotReported)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(not_reported.len(), 2);
    assert!(not_reported.contains(&sub_1));
    assert!(not_reported.contains(&sub_3));

    let reported = model
        .fetch_all_by_status(Status::Reported)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(reported.len(), 1);
    assert!(reported.contains(&sub_2));
}

#[tokio::test]
async fn test_subscription_model_get_reported_date_range() {
    let db_pool = get_test_db_pool().await;
    let model = SubscriptionModel { db_pool: &db_pool };
    // Sub 1 should not be included in the date range
    let mut sub_1 = make_fake_sub();
    sub_1.update_status(Status::NotReported);
    sub_1.set_status_t(Some(sub_1.get_status_t().unwrap() - Duration::hours(100)));
    // Sub 2 - this is the max
    let mut sub_2 = make_fake_sub();
    sub_2.update_status(Status::Reported);
    // Sub 3 - this is the min
    let mut sub_3 = make_fake_sub();
    sub_3.update_status(Status::Reported);
    sub_3.set_status_t(Some(sub_3.get_status_t().unwrap() - Duration::hours(10)));
    // Sub 4 should not be included in the date range
    let mut sub_4 = make_fake_sub();
    sub_4.update_status(Status::NotReported);
    sub_4.set_status_t(Some(sub_4.get_status_t().unwrap() + Duration::hours(100)));

    for sub in [&sub_1, &sub_2, &sub_3, &sub_4] {
        model
            .create_from_sub(sub)
            .await
            .expect("Failed to create sub.");
    }

    let all = model.fetch_all().await.unwrap();
    assert_eq!(all.len(), 4);
    let result = model
        .get_reported_date_range()
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(
        result.max.unwrap().unix_timestamp(),
        sub_2.get_status_t().unwrap().unix_timestamp()
    );
    assert_eq!(
        result.min.unwrap().unix_timestamp(),
        sub_3.get_status_t().unwrap().unix_timestamp()
    );
}

#[tokio::test]
async fn test_subscription_update_sub_status() {
    let db_pool = get_test_db_pool().await;
    let model = SubscriptionModel { db_pool: &db_pool };
    let sub = make_fake_sub();
    save_sub(&model, &sub).await;
    assert_eq!(sub.get_status_history().unwrap().entries.len(), 1);
    model
        .update_sub_status(&sub.id, Status::WillNotReport)
        .await
        .expect("Should not fail.");
    let result = model.fetch_one_by_id(&sub.id).await.unwrap();
    assert_eq!(result.get_status().unwrap(), Status::WillNotReport);
    let result_status_history = result.get_status_history().unwrap();
    assert_eq!(result_status_history.entries.len(), 2);
    assert_eq!(
        result_status_history.entries[1].status,
        Status::WillNotReport
    );
    // This won't pass if the test is slower than a second to process
    assert_eq!(
        result_status_history.entries[1].t.unix_timestamp(),
        OffsetDateTime::now_utc().unix_timestamp()
    );
    // Go again after a delay updating to Reported
    std::thread::sleep(std::time::Duration::from_secs(2));
    model
        .update_sub_status(&sub.id, Status::Reported)
        .await
        .expect("Should not fail.");
    let result = model.fetch_one_by_id(&sub.id).await.unwrap();
    assert_eq!(result.get_status().unwrap(), Status::Reported);
    let result_status_history = result.get_status_history().unwrap();
    assert_eq!(result_status_history.entries.len(), 3);
    assert_eq!(result_status_history.entries[2].status, Status::Reported);
    assert_eq!(
        result_status_history.entries[2].t.unix_timestamp(),
        OffsetDateTime::now_utc().unix_timestamp()
    );
}
