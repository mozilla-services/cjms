use sqlx::{Pool, Postgres};
use time::OffsetDateTime;

use crate::models::{
    refunds::RefundModel,
    status_history::{Status, UpdateStatus},
};

pub async fn batch_refunds_by_day(db_pool: &Pool<Postgres>) {
    let refunds = RefundModel { db_pool };
    // Intentional panic. Cannot continue if we can't retrieve refunds.
    let not_reported_refunds = refunds
        .fetch_all_not_reported()
        .await
        .expect("Could not retrieve refunds from DB.");
    println!("Found {} refunds to report.", not_reported_refunds.len());
    for mut refund in not_reported_refunds {
        refund.correction_file_date = Some(OffsetDateTime::now_utc().date());
        refund.update_status(Status::Reported);
        match refunds.update_refund(&refund).await {
            Ok(r) => {
                println!("Success updating refund: {}", &r.refund_id);
            }
            Err(e) => {
                println!(
                    "Could not update refund {} to be reported. {}",
                    &refund.refund_id, e
                );
            }
        };
    }
}
