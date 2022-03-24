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
        match cj_client.report_subscription(&sub).await {
            Ok(r) => {
                if r.status() == 200 {
                    match subscriptions
                        .update_sub_status(&sub.id, Status::Reported)
                        .await
                    {
                        Ok(_) => {
                            // TODO - LOGGING
                            println!("Success for sub: {}", sub.flow_id);
                        }
                        Err(e) => {
                            // TODO - I'm not sure how we want to handle this situation
                            println!(
                                "Could not mark subscription as reported. But it has been. {}",
                                e
                            );
                        }
                    };
                }

                // TODO - Make subscriptions model function for marking "not reporting and appending to history and also use in check subscriptions"
                // mark not reported
            }
            Err(e) => {
                // TODO - LOGGING
                println!("R {}", e);
                // mark not reported
            }
        }
    }
}
