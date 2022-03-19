use actix_web::cookie::time::OffsetDateTime;
use serde_json::json;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    bigquery::client::{BQClient, BQError, ResultSet},
    models::{
        aic::AICModel,
        subscriptions::{Subscription, SubscriptionModel},
    },
};

// Throw an error if required fields are not available
fn make_subscription_from_bq_row(rs: &ResultSet) -> Result<Subscription, BQError> {
    let sub = Subscription {
        id: Uuid::new_v4(),
        flow_id: rs.require_string_by_name("flow_id")?,
        subscription_id: rs.require_string_by_name("subscription_id")?,
        report_timestamp: rs.require_offsetdatetime_by_name("report_timestamp")?,
        subscription_created: rs.require_offsetdatetime_by_name("subscription_created")?,
        fxa_uid: rs.require_string_by_name("fxa_uid")?,
        quantity: rs.require_i32_by_name("quantity")?,
        plan_id: rs.require_string_by_name("plan_id")?,
        plan_currency: rs.require_string_by_name("plan_currency")?,
        plan_amount: rs.require_i32_by_name("plan_amount")?,
        country: rs.get_string_by_name("country")?,
        aic_id: None,
        aic_expires: None,
        cj_event_value: None,
        status: None,
        status_history: None,
    };
    Ok(sub)
}

pub async fn fetch_and_process_new_subscriptions(bq: BQClient, db_pool: &Pool<Postgres>) {
    let subscriptions = SubscriptionModel { db_pool };
    let aics = AICModel { db_pool };
    // Get all results from bigquery table that stores new subscription reports
    let query = "SELECT * FROM `cjms_bigquery.cj_attribution_v1`;";
    let mut rs = bq.get_bq_results(query).await;
    while rs.next_row() {
        // If can't deserialize e.g. required fields are not available log and move on.
        let mut sub = match make_subscription_from_bq_row(&rs) {
            Ok(sub) => sub,
            Err(e) => {
                // TODO - LOGGING - Log information and get a metric
                println!(
                    "Failed to make subscription for bq result row. {:?}. Continuing...",
                    e
                );
                continue;
            }
        };
        let aic = match aics.fetch_one_by_flow_id(&sub.flow_id).await {
            Ok(aic) => aic,
            Err(_) => match aics.fetch_one_by_flow_id_from_archive(&sub.flow_id).await {
                Ok(aic) => {
                    // TODO - LOGGING - Note that we had to pull from archive table
                    println!("AIC was retrieved from archive table");
                    aic
                }
                Err(e) => {
                    // TODO - LOGGING
                    println!(
                        "Errorr getting aic for subscription: {:?}. Continuing....",
                        e
                    );
                    continue;
                }
            },
        };
        sub.aic_id = Some(aic.id);
        sub.cj_event_value = Some(aic.cj_event_value.clone());
        sub.aic_expires = Some(aic.expires);
        sub.status = Some("not_reported".to_string());
        sub.status_history = Some(json!([{
            "status": "not_reported",
            "t": OffsetDateTime::now_utc().to_string()
        }]));
        // Archive the AIC
        match aics.archive_aic(&aic).await {
            Ok(_) => {}
            Err(e) => {
                // TODO - LOGGING
                println!("Failed to archive aic entry: {:?}. Continuing...", e);
                continue;
            }
        };
        // Save the new subscription entry
        match subscriptions.create_from_sub(&sub).await {
            Ok(sub) => sub,
            Err(e) => match e {
                sqlx::Error::Database(e) => {
                    // 23505 is the code for unique constraints e.g. duplicate flow id issues
                    if e.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                        // TODO - LOGGING - add some specific logging / metrics around duplicate key issues.
                        // This could help us see that we have an ETL issue.
                        println!("Duplicate Key Violation");
                    }
                    println!(
                        "DatabaseError error while creating subscription {:?}. Continuing...",
                        e
                    );
                    continue;
                }
                _ => {
                    println!(
                        "Unexpected error while creating subscription {:?}. Continuing...",
                        e
                    );
                    continue;
                }
            },
        };
    }
}
