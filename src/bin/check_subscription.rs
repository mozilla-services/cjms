use lib::{check_subscriptions::get_bqclient, settings::get_settings};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = get_settings();
    let _ = get_bqclient(&settings);
    Ok(())
}
