use lib::{
    jobs::verify_reports::verify_reports_with_cj,
    settings::get_settings,
    telemetry::StatsD,
    models::{
        status_history::{Status, StatusHistoryEntry, UpdateStatus},
        subscriptions::SubscriptionModel,
    },
    cj::{client::CJClient},
};
use time::OffsetDateTime;
use wiremock::{
    matchers::{method, path, query_param},
    Mock, MockBuilder, MockServer, ResponseTemplate,
};
use crate::{models::refunds::make_fake_refund, models::subscriptions::make_fake_sub, utils::get_test_db_pool};

#[tokio::test]
async fn batch_refunds_by_day_makes_unreported_subscriptions_reported_and_gives_a_day() {
    // SETUP
    let settings = get_settings();
    let mock_statsd = StatsD::new(&settings);
    let db_pool = get_test_db_pool().await;
    let sub_model = SubscriptionModel { db_pool: &db_pool };

    // Sub 1 - should be reported
    let mut sub_1 = make_fake_sub();

    for sub in [&sub_1,] {
        sub_model
            .create_from_sub(sub)
            .await
            .expect("Failed to create sub.");
    }
    let mock_cj = MockServer::start().await;
    let mock_cj_client = CJClient::new(&settings, None, Some(&mock_cj.uri()));
}
