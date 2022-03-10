use std::env;

use cjms::actions::bigquery;
use cjms::actions::subscription::check_subscriptions;
use cjms::appconfig::connect_to_database;
use cjms::settings::get_settings;
use time::OffsetDateTime;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Run command with access_type specified `./check_subscription env` or `./check_subscription metadata`.")
    }
    let access_type = &args[1];
    let bq_access_token = match access_type.as_str() {
        "env" => bigquery::get_access_token_from_env().await,
        "metadata" => bigquery::get_access_token_from_metadata().await,
        _ => {
            panic!("Run command with access_type specified `./check_subscription env` or `./check_subscription metadata`.")
        }
    };
    println!("Running check subscriptions bin.");
    println!("Start time: {}", OffsetDateTime::now_utc());
    let settings = get_settings();
    let db_pool = connect_to_database(&settings.database_url).await;
    check_subscriptions(&db_pool, bq_access_token).await;
    db_pool.close().await;
    println!("End time: {}", OffsetDateTime::now_utc());
    Ok(())
}
