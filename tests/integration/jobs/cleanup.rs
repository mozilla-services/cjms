use lib::models::aic::AICModel;

use crate::utils::get_test_db_pool;

#[tokio::test]
async fn archive_expired_aics() {
    // SETUP
    let db_pool = get_test_db_pool().await;
    let _aics = AICModel { db_pool: &db_pool };
}
