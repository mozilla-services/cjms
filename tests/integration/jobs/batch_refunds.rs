use lib::{
    jobs::batch_refunds::batch_refunds_by_day,
    models::{
        refunds::RefundModel,
        status_history::{Status, StatusHistoryEntry, UpdateStatus},
    },
    settings::get_settings,
    telemetry::StatsD,
};
use time::OffsetDateTime;

use crate::{models::refunds::make_fake_refund, utils::get_test_db_pool};

#[tokio::test]
async fn batch_refunds_by_day_makes_unreported_subscriptions_reported_and_gives_a_day() {
    // SETUP
    let settings = get_settings();
    let mock_statsd = StatsD::new(&settings);
    let db_pool = get_test_db_pool().await;
    let refunds = RefundModel { db_pool: &db_pool };

    // Refund 1,2 - should be reported (refund_status None | succeeded)
    let mut r_1 = make_fake_refund();
    r_1.refund_status = None;
    let mut r_2 = make_fake_refund();
    r_2.refund_status = Some("succeeded".to_string());
    // Refund 3,4,5 - should be WillNotReport (refund_status pending | failed | canceled)
    let mut r_3 = make_fake_refund();
    r_3.refund_status = Some("pending".to_string());
    let mut r_4 = make_fake_refund();
    r_4.refund_status = Some("failed".to_string());
    let mut r_5 = make_fake_refund();
    r_5.refund_status = Some("canceled".to_string());

    for refund in [&r_1, &r_2, &r_3, &r_4, &r_5] {
        refunds
            .create_from_refund(refund)
            .await
            .expect("Failed to create refund.");
    }

    // GO
    std::thread::sleep(std::time::Duration::from_secs(2));
    let now = OffsetDateTime::now_utc();
    batch_refunds_by_day(&db_pool, &mock_statsd).await;

    // ASSERT

    let r_1_updated = refunds
        .fetch_one_by_refund_id(&r_1.refund_id)
        .await
        .expect("Could not get refund");
    let r_2_updated = refunds
        .fetch_one_by_refund_id(&r_2.refund_id)
        .await
        .expect("Could not get refund");
    let r_3_updated = refunds
        .fetch_one_by_refund_id(&r_3.refund_id)
        .await
        .expect("Could not get refund");
    let r_4_updated = refunds
        .fetch_one_by_refund_id(&r_4.refund_id)
        .await
        .expect("Could not get refund");
    let r_5_updated = refunds
        .fetch_one_by_refund_id(&r_5.refund_id)
        .await
        .expect("Could not get refund");

    for refund_updated in [&r_1_updated, &r_2_updated] {
        assert_eq!(refund_updated.correction_file_date.unwrap(), now.date());
        assert_eq!(refund_updated.get_status().unwrap(), Status::Reported);
        let refund_updated_history = refund_updated.get_status_history().unwrap();
        assert_eq!(refund_updated_history.entries.len(), 2);
        assert_eq!(
            refund_updated_history.entries[1],
            StatusHistoryEntry {
                status: Status::Reported,
                t: now
            }
        );
    }
    for refund_updated in [&r_3_updated, &r_4_updated, &r_5_updated] {
        assert!(refund_updated.correction_file_date.is_none());
        assert_eq!(refund_updated.get_status().unwrap(), Status::WillNotReport);
        let refund_updated_history = refund_updated.get_status_history().unwrap();
        assert_eq!(refund_updated_history.entries.len(), 2);
        assert_eq!(
            refund_updated_history.entries[1],
            StatusHistoryEntry {
                status: Status::WillNotReport,
                t: now
            }
        );
    }
}
