use time::OffsetDateTime;
fn main() -> std::io::Result<()> {
    println!("Running check subscriptions bin.");
    println!("Start time: {}", OffsetDateTime::now_utc());
    println!("End time: {}", OffsetDateTime::now_utc());
    Ok(())
}
