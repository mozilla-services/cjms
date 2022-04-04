use lib::{
    appconfig::connect_to_database_and_migrate, jobs::batch_refunds::batch_refunds_by_day,
    settings::get_settings,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = get_settings();
    let db = connect_to_database_and_migrate(&settings.database_url).await;
    // TODO - LOGGING - This is a process we'll want to log and time (if possible)
    println!("Starting batch_refunds_by_day");
    batch_refunds_by_day(&db).await;
    println!("End batch_refunds_by_day");
    db.close().await;
    Ok(())
}
