use cjms::controllers::custodial::VERSION_FILE;
use std::fs;
use std::str::from_utf8;
use std::process::Command;

fn main() -> std::io::Result<()> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output()
        .expect("failed to execute process");
    let result = from_utf8(&output.stdout).expect("Failed to read output.");
    println!("{:?}", result);
    fs::write(VERSION_FILE, "test").ok();
    Ok(())
}
