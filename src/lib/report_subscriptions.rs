use sqlx::{Pool, Postgres};

use crate::{
    cj::client::CJS2SClient,
    models::subscriptions::{Status, SubscriptionModel},
};

// TODO - LOGGING
// Need logging through out all these possible match conditions. Marking each println with the tag seemed redundant.

pub async fn report_subscriptions_to_cj(db_pool: &Pool<Postgres>, cj_client: CJS2SClient) {
    let subscriptions = SubscriptionModel { db_pool };
    // Intentional panic. Cannot continue if we can't retrieve subs.
    let not_reported_subscriptions = subscriptions
        .fetch_all_not_reported()
        .await
        .expect("Could not retrieve subscriptions from DB.");
    for sub in not_reported_subscriptions {
        let next_status = match sub.aic_expires {
            Some(aic_expires) => {
                if aic_expires < sub.subscription_created {
                    println!(
                        "Affiliate cookie expired before subscription {} created. Will not report.",
                        &sub.id
                    );
                    Status::WillNotReport
                } else {
                    Status::Reported
                }
            }
            None => {
                println!(
                    "Sub {} does not have an aic expires. Will not report.",
                    &sub.id
                );
                Status::WillNotReport
            }
        };
        if next_status == Status::WillNotReport {
            match subscriptions
                .update_sub_status(&sub.id, Status::WillNotReport)
                .await
            {
                Ok(_) => {
                    println!("Successfully marked as WillNotReport.");
                }
                Err(e) => {
                    println!(
                        "Could not mark subscription {} as WillNotReport. {}",
                        &sub.id, e
                    );
                }
            };
            continue;
        }

        let mark_not_reported = match cj_client.report_subscription(&sub).await {
            Ok(r) => {
                if r.status() == 200 {
                    match subscriptions
                        .update_sub_status(&sub.id, Status::Reported)
                        .await
                    {
                        Ok(_) => {
                            println!("200 Success for sub: {}", &sub.id);
                        }
                        Err(e) => {
                            println!(
                                "Could not mark subscription {} as reported. But it has been. {}",
                                &sub.id, e
                            );
                        }
                    };
                    false
                } else {
                    println!(
                        "Not 200 Success for sub: {}. Marking Not Reported.",
                        &sub.id
                    );
                    true
                }
            }
            Err(e) => {
                println!(
                    "Report_subscription errored. Marking sub {} not reported. {}",
                    &sub.id, e
                );
                true
            }
        };
        if mark_not_reported {
            match subscriptions
                .update_sub_status(&sub.id, Status::NotReported)
                .await
            {
                Ok(_) => {
                    println!("Successfully marked as NotReported.");
                }
                Err(e) => {
                    println!(
                        "Could not mark subscription {} as not_reported. {}",
                        &sub.id, e
                    );
                }
            }
        }
    }
}
