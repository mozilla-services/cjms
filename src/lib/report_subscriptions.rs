use sqlx::{Pool, Postgres};

use crate::{
    cj::client::CJS2SClient,
    models::subscriptions::{Status, SubscriptionModel},
};

pub async fn report_subscriptions_to_cj(db_pool: &Pool<Postgres>, cj_client: CJS2SClient) {
    let subscriptions = SubscriptionModel { db_pool };
    // TODO - no unwrap
    let not_reported_subscriptions = subscriptions.fetch_all_not_reported().await.unwrap();
    for sub in not_reported_subscriptions {
        println!("Working on sub: {}", &sub.flow_id);
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
                // TODO - LOGGING
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
                    // TODO - LOGGING
                }
                Err(e) => {
                    // TODO - LOGGING
                    println!(
                        "Could not mark subscription {} as WillNotReport. {}",
                        &sub.id, e
                    );
                }
            };
            continue;
        }

        match cj_client.report_subscription(&sub).await {
            Ok(r) => {
                if r.status() == 200 {
                    match subscriptions
                        .update_sub_status(&sub.id, Status::Reported)
                        .await
                    {
                        Ok(_) => {
                            // TODO - LOGGING
                            // TODO - change to id
                            println!("200 Success for sub: {}", &sub.id);
                        }
                        Err(e) => {
                            // TODO - LOGGING
                            println!(
                                "Could not mark subscription {} as reported. But it has been. {}",
                                &sub.id, e
                            );
                        }
                    };
                } else {
                    // TODO - LOGGING
                    // TODO - change to id
                    println!("Not 200 Success for sub: {}", &sub.flow_id);
                    match subscriptions
                        .update_sub_status(&sub.id, Status::NotReported)
                        .await
                    {
                        Ok(_) => {
                            // TODO - LOGGING
                            println!("CJ did not return a 200 for sub: {}", &sub.id);
                        }
                        Err(e) => {
                            // TODO - LOGGING
                            println!(
                                "Could not mark subscription {} as not_reported. {}",
                                &sub.id, e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                // TODO - LOGGING
                println!("R {}", e);
                // mark not reported
            }
        }
    }
}
