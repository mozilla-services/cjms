use crate::utils::{get_test_db_pool, random_price, random_simple_ascii_string};
use lib::models::refunds::{PartialRefund, Refund, RefundModel};
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
async fn test_refund_model_create_from_refund_struct() {
    let db_pool = get_test_db_pool().await;
    let model = RefundModel { db_pool: &db_pool };
    let r = make_fake_refund();
    save_refund(&model, &r).await;
    let result = model
        .fetch_one_by_id(&r.id)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(result, r);
}
