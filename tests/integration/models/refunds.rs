use crate::utils::{get_test_db_pool, random_price, random_simple_ascii_string};
use lib::models::{
    refunds::{PartialRefund, Refund, RefundModel},
    status_history::{Status, UpdateStatus},
};
use pretty_assertions::assert_eq;
use time::OffsetDateTime;
use uuid::Uuid;

pub fn make_fake_refund() -> Refund {
    Refund::new(PartialRefund {
        id: Uuid::new_v4(),
        subscription_id: random_simple_ascii_string(),
        refund_id: random_simple_ascii_string(),
        refund_created: OffsetDateTime::now_utc(),
        refund_amount: random_price(),
        refund_status: Some(random_simple_ascii_string()),
        refund_reason: Some(random_simple_ascii_string()),
    })
}

pub async fn save_refund(model: &RefundModel<'_>, refund: &Refund) {
    model
        .create_from_refund(refund)
        .await
        .expect("Failed to create test object.");
}

#[tokio::test]
async fn test_refund_model_create_from_refund_and_fetch_by_refund_id() {
    let db_pool = get_test_db_pool().await;
    let model = RefundModel { db_pool: &db_pool };
    let r = make_fake_refund();
    save_refund(&model, &r).await;
    let result = model
        .fetch_one_by_refund_id(&r.refund_id)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(result, r);
}

#[tokio::test]
async fn test_refund_model_update_refund() {
    let db_pool = get_test_db_pool().await;
    let model = RefundModel { db_pool: &db_pool };
    let r = make_fake_refund();
    let refund_id = r.refund_id.clone();
    save_refund(&model, &r).await;
    let mut r_update = make_fake_refund();
    r_update.refund_id = refund_id.clone();
    r_update.id = r.id;
    model
        .update_refund(&mut r_update)
        .await
        .expect("Failed to update refund.");
    let result = model
        .fetch_one_by_refund_id(&refund_id)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(r_update, result);
}

#[tokio::test]
async fn test_refund_model_update_refund_updates_the_status_to_not_reported() {
    let db_pool = get_test_db_pool().await;
    let model = RefundModel { db_pool: &db_pool };
    let mut r = make_fake_refund();
    r.update_status(Status::Reported);
    r.set_raw_status_history(None);
    save_refund(&model, &r).await;
    std::thread::sleep(std::time::Duration::from_secs(2));
    let mut r_update = make_fake_refund();
    r_update.refund_id = r.refund_id.clone();
    // This has no effect. It helps us verify that the update_refund function is setting the latest
    // status to NotReported.
    r_update.update_status(Status::WillNotReport);
    let now = OffsetDateTime::now_utc();
    model
        .update_refund(&mut r_update)
        .await
        .expect("Failed to update refund.");
    let result = model
        .fetch_one_by_refund_id(&r.refund_id)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(result.get_status().unwrap(), Status::NotReported);
    assert_eq!(result.get_status_t().unwrap().unix_timestamp(), now.unix_timestamp());
    assert_eq!(
        result.get_status_history().unwrap().entries[1]
            .t
            .unix_timestamp(),
        now.unix_timestamp()
    );
}
