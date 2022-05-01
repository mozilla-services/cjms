use crate::utils::{get_test_db_pool, random_price, random_simple_ascii_string};
use lib::models::{
    refunds::{PartialRefund, Refund, RefundModel},
    status_history::{Status, UpdateStatus},
};
use pretty_assertions::assert_eq;
use time::{date, Duration, OffsetDateTime};
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
        correction_file_date: None,
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
        .update_refund(&r_update)
        .await
        .expect("Failed to update refund.");
    let result = model
        .fetch_one_by_refund_id(&refund_id)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(r_update, result);
}

#[tokio::test]
async fn test_refund_model_fetch_all_by_status() {
    let db_pool = get_test_db_pool().await;
    let model = RefundModel { db_pool: &db_pool };
    let refund_1 = make_fake_refund();
    save_refund(&model, &refund_1).await;

    let mut refund_2 = make_fake_refund();
    refund_2.update_status(Status::Reported);
    save_refund(&model, &refund_2).await;

    let refund_3 = make_fake_refund();
    save_refund(&model, &refund_3).await;

    // Null status_t should not be returned
    let mut refund_4 = make_fake_refund();
    refund_4.set_status_t(None);
    save_refund(&model, &refund_4).await;

    let all = model.fetch_all().await.unwrap();
    assert_eq!(all.len(), 4);

    let not_reported = model
        .fetch_all_by_status(Status::NotReported)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(not_reported.len(), 2);
    assert!(not_reported.contains(&refund_1));
    assert!(not_reported.contains(&refund_3));

    let reported = model
        .fetch_all_by_status(Status::Reported)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(reported.len(), 1);
    assert!(reported.contains(&refund_2));
}

#[tokio::test]
async fn test_refundscription_model_get_reported_date_range() {
    let db_pool = get_test_db_pool().await;
    let model = RefundModel { db_pool: &db_pool };
    // Refund 1 should not be included in the date range
    let mut refund_1 = make_fake_refund();
    refund_1.update_status(Status::NotReported);
    refund_1.set_status_t(Some(
        refund_1.get_status_t().unwrap() - Duration::hours(100),
    ));
    // refund 2 - this is the max
    let mut refund_2 = make_fake_refund();
    refund_2.update_status(Status::Reported);
    // refund 3 - this is the min
    let mut refund_3 = make_fake_refund();
    refund_3.update_status(Status::Reported);
    refund_3.set_status_t(Some(refund_3.get_status_t().unwrap() - Duration::hours(10)));
    // refund 4 should not be included in the date range
    let mut refund_4 = make_fake_refund();
    refund_4.update_status(Status::NotReported);
    refund_4.set_status_t(Some(
        refund_4.get_status_t().unwrap() + Duration::hours(100),
    ));

    for refund in [&refund_1, &refund_2, &refund_3, &refund_4] {
        model
            .create_from_refund(refund)
            .await
            .expect("Failed to create refund.");
    }

    let all = model.fetch_all().await.unwrap();
    assert_eq!(all.len(), 4);
    let result = model
        .get_reported_date_range()
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(
        result.max.unwrap().unix_timestamp(),
        refund_2.get_status_t().unwrap().unix_timestamp()
    );
    assert_eq!(
        result.min.unwrap().unix_timestamp(),
        refund_3.get_status_t().unwrap().unix_timestamp()
    );
}

#[tokio::test]
async fn test_refund_model_get_by_correction_file_day() {
    let db_pool = get_test_db_pool().await;
    let model = RefundModel { db_pool: &db_pool };

    let today = date!(2022 - 02 - 03);
    let another_day = date!(2021 - 11 - 01);

    let mut refund_1 = make_fake_refund();
    refund_1.correction_file_date = Some(today);
    let mut refund_2 = make_fake_refund();
    refund_2.correction_file_date = Some(another_day);
    let mut refund_3 = make_fake_refund();
    refund_3.correction_file_date = Some(today);
    let mut refund_4 = make_fake_refund();
    refund_4.correction_file_date = None;

    for r in [&refund_1, &refund_2, &refund_3, &refund_4] {
        save_refund(&model, r).await;
    }

    let all = model.fetch_all().await.unwrap();
    assert_eq!(all.len(), 4);
    let result = model
        .fetch_by_correction_file_day(&today)
        .await
        .expect("Could not fetch from DB.");
    assert_eq!(result.len(), 2);
    assert!(result.contains(&refund_1));
    assert!(result.contains(&refund_3));
}

#[tokio::test]
async fn test_refund_update_refund_status() {
    let db_pool = get_test_db_pool().await;
    let model = RefundModel { db_pool: &db_pool };
    let refund = make_fake_refund();
    save_refund(&model, &refund).await;
    assert_eq!(refund.get_status_history().unwrap().entries.len(), 1);
    model
        .update_refund_status(&refund.refund_id, Status::CJReceived)
        .await
        .expect("Should not fail.");
    let result = model
        .fetch_one_by_refund_id(&refund.refund_id)
        .await
        .unwrap();
    assert_eq!(result.get_status().unwrap(), Status::CJReceived);
    let result_status_history = result.get_status_history().unwrap();
    assert_eq!(result_status_history.entries.len(), 2);
    assert_eq!(result_status_history.entries[1].status, Status::CJReceived);
    // This won't pass if the test is slower than a second to process
    assert_eq!(
        result_status_history.entries[1].t.unix_timestamp(),
        OffsetDateTime::now_utc().unix_timestamp()
    );
    // Go again after a delay updating to Reported
    std::thread::sleep(std::time::Duration::from_secs(2));
    model
        .update_refund_status(&refund.refund_id, Status::Reported)
        .await
        .expect("Should not fail.");
    let result = model
        .fetch_one_by_refund_id(&refund.refund_id)
        .await
        .unwrap();
    assert_eq!(result.get_status().unwrap(), Status::Reported);
    let result_status_history = result.get_status_history().unwrap();
    assert_eq!(result_status_history.entries.len(), 3);
    assert_eq!(result_status_history.entries[2].status, Status::Reported);
    assert_eq!(
        result_status_history.entries[2].t.unix_timestamp(),
        OffsetDateTime::now_utc().unix_timestamp()
    );
}
