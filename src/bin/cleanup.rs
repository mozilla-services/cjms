use lib::{
    appconfig::connect_to_database_and_migrate, jobs::cleanup::archive_expired_aics,
    settings::get_settings,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = get_settings();
    let db = connect_to_database_and_migrate(&settings.database_url).await;
    // TODO - LOGGING - This is a process we'll want to log and time (if possible)
    println!("Starting cleanup");
    archive_expired_aics(&db).await;
    println!("End cleanup");
    db.close().await;
    Ok(())
}
