use cjms::appconfig::connect_to_database;
use cjms::controllers::subscription::check_subscriptions;
use cjms::settings::get_settings;
use time::OffsetDateTime;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Running check subscriptions bin.");
    println!("Start time: {}", OffsetDateTime::now_utc());
    let settings = get_settings();
    let db_pool = connect_to_database(&settings.database_url).await;
    check_subscriptions(&db_pool).await;
    db_pool.close().await;
    println!("End time: {}", OffsetDateTime::now_utc());
    Ok(())
}
