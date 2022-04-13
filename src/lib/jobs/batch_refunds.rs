use sqlx::{Pool, Postgres};
use time::OffsetDateTime;

use crate::{
    models::{
        refunds::RefundModel,
        status_history::{Status, UpdateStatus},
    },
    telemetry::{StatsD, TraceType},
};

pub async fn batch_refunds_by_day(db_pool: &Pool<Postgres>, statsd: &StatsD) {
    let refunds = RefundModel { db_pool };
    // Intentional panic. Cannot continue if we can't retrieve refunds.
    let not_reported_refunds = refunds
        .fetch_not_reported()
        .await
        .expect("Could not retrieve refunds from DB.");
    statsd.gauge(
        &TraceType::BatchRefunds,
        "n-not-reported",
        not_reported_refunds.len(),
    );
    for mut refund in not_reported_refunds {
        let next_state = match &refund.refund_status {
            Some(refund_status) => {
                if refund_status == "succeeded" {
                    Status::Reported
                } else {
                    Status::WillNotReport
                }
            }
            None => Status::Reported,
        };
        if next_state == Status::Reported {
            refund.correction_file_date = Some(OffsetDateTime::now_utc().date());
        }
        refund.update_status(next_state);
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
