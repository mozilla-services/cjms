use lib::{appconfig::connect_to_database_and_migrate, settings::get_settings};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = get_settings();
    let db = connect_to_database_and_migrate(&settings.database_url).await;
    // TODO - LOGGING - This is a process we'll want to log and time (if possible)
    println!("Starting report_subscriptions");
    //report_subscriptions_to_cj(&db).await;
    println!("End report_subscriptions");
    db.close().await;
    Ok(())
}
