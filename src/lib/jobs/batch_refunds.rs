use sqlx::{Pool, Postgres};
use time::OffsetDateTime;

use crate::{
    error_and_incr, info_and_incr,
    models::{
        refunds::RefundModel,
        status_history::{Status, UpdateStatus},
    },
    telemetry::{LogKey, StatsD},
};

pub async fn batch_refunds_by_day(db_pool: &Pool<Postgres>, statsd: &StatsD) {
    let refunds = RefundModel { db_pool };
    // Intentional panic. Cannot continue if we can't retrieve refunds.
    let not_reported_refunds = refunds
        .fetch_all_by_status(Status::NotReported)
        .await
        .expect("Could not retrieve refunds from DB.");
    statsd.gauge(
        &LogKey::BatchRefundsNNotReported,
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
                info_and_incr!(
                    statsd,
                    LogKey::BatchRefundsUpdate,
                    refund_id = &r.refund_id.as_str(),
                    "Success updating refund"
                );
            }
            Err(e) => {
                error_and_incr!(
                    statsd,
                    LogKey::BatchRefundsUpdateFailed,
                    error = e,
                    refund_id = &refund.refund_id.as_str(),
                    "Could not update refund to be reported"
                );
            }
        };
    }
}
