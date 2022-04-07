use lib::{
    appconfig::connect_to_database_and_migrate,
    bigquery::client::get_bqclient,
    jobs::check_refunds::fetch_and_process_refunds,
    settings::get_settings,
    telemetry::{init_sentry, init_tracing},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = get_settings();

    let _guard = init_sentry(&settings);
    init_tracing("cjms-check-refunds", &settings.log_level, std::io::stdout);

    let bq = get_bqclient(&settings).await;
    let db = connect_to_database_and_migrate(&settings.database_url).await;
    // TODO - LOGGING - This is a process we'll want to log and time (if possible)
    println!("Starting fetch_and_process_refunds");
    fetch_and_process_refunds(bq, &db).await;
    println!("End fetch_and_process_refunds");
    db.close().await;
    Ok(())
}
