use lib::{
    appconfig::connect_to_database_and_migrate, bigquery::client::get_bqclient,
    check_subscriptions::fetch_and_process_new_subscriptions, settings::get_settings,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = get_settings();
    let bq = get_bqclient(&settings).await;
    let db = connect_to_database_and_migrate(&settings.database_url).await;
    // TODO - Jeremy this is a process we'll want to log and time (if possible)
    println!("Starting fetch_and_process_new_subscriptions");
    fetch_and_process_new_subscriptions(bq, &db).await;
    println!("End fetch_and_process_new_subscriptions");
    db.close().await;
    Ok(())
}
