use lib::{
    jobs::batch_refunds::batch_refunds_by_day,
    models::{
        refunds::RefundModel,
        status_history::{Status, StatusHistoryEntry, UpdateStatus},
    },
};
use time::OffsetDateTime;

use crate::{models::refunds::make_fake_refund, utils::get_test_db_pool};

#[tokio::test]
async fn batch_refunds_by_day_makes_unreported_subscriptions_reported_and_gives_a_day() {
    // SETUP

    let db_pool = get_test_db_pool().await;
    let refunds = RefundModel { db_pool: &db_pool };

    // Refund 1 - should be reported
    let r_1 = make_fake_refund();

    for refund in [&r_1] {
        refunds
            .create_from_refund(refund)
            .await
            .expect("Failed to create refund.");
    }

    // GO
    std::thread::sleep(std::time::Duration::from_secs(2));
    let now = OffsetDateTime::now_utc();
    batch_refunds_by_day(&db_pool).await;

    // ASSERT

    let r_1_updated = refunds
        .fetch_one_by_refund_id(&r_1.refund_id)
        .await
        .expect("Could not get refund");
    assert_eq!(r_1_updated.correction_file_date.unwrap(), now.date());
    assert_eq!(r_1_updated.get_status().unwrap(), Status::Reported);
    let r_1_updated_history = r_1_updated.get_status_history().unwrap();
    assert_eq!(r_1_updated_history.entries.len(), 2);
    assert_eq!(
        r_1_updated_history.entries[1],
        StatusHistoryEntry {
            status: Status::Reported,
            t: now
        }
    );
}
