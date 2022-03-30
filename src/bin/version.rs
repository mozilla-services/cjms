use lib::{controllers::custodial::VERSION_FILE, settings::get_settings, telemetry::init_tracing};
use std::process::Command;
use std::str;
use std::{env, fs};

fn main() -> std::io::Result<()> {
    let settings = get_settings();
    init_tracing("cjms-version", &settings.log_level, std::io::stdout);
    write_version_file();
    Ok(())
}

fn write_version_file() -> std::io::Result<()> {
    let (sha, tag) = match env::var("CI") {
        Ok(_) => {
            // If we're in CI use local variables
            let sha = env::var("GITHUB_SHA").expect("Missing environment variable GITHUB_SHA");
            let tag =
                env::var("GITHUB_REF_NAME").expect("Missing environment variable GITHUB_REF_NAME");
            (sha, tag)
        }
        Err(_) => {
            // Commit SHA
            let rev_parse_out = Command::new("git")
                .arg("rev-parse")
                .arg("--short")
                .arg("HEAD")
                .output()
                .expect("Failed to execute git rev-parse");
            let sha =
                str::trim(str::from_utf8(&rev_parse_out.stdout).expect("Failed to read output."))
                    .to_string();
            // Version
            let tags_out = Command::new("git")
                .arg("describe")
                .arg("--tags")
                .output()
                .expect("Failed to execute git rev-parse");
            let tag = str::trim(str::from_utf8(&tags_out.stdout).expect("Failed to read output."))
                .to_string();
            (sha, tag)
        }
    };
    // Repo link
    let source = env!("CARGO_PKG_REPOSITORY");
    fs::write(
        VERSION_FILE,
        format!("source: {}\ncommit: {}\nversion: {}\n", source, sha, tag),
    )
    .expect("Failed to write file.");
    Ok(())
}

#[cfg(test)]
mod test_bin_version {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_writes_a_new_file_that_has_source() {
        fs::remove_file(VERSION_FILE).ok();
        write_version_file().expect("Couldn't run version binary.");
        let file_stream = fs::read(VERSION_FILE).expect("Couldn't read version file");
        let version_file = str::from_utf8(&file_stream).expect("Couldn't read from version file.");
        let source = env!("CARGO_PKG_REPOSITORY");
        let error_msg = format!("Got version file contents: \n{}", version_file);
        assert!(
            version_file.contains(format!("source: {}", source).as_str()),
            "{}",
            error_msg
        );
        assert!(version_file.contains("commit:"), "{}", error_msg);
        assert!(version_file.contains("version:"), "{}", error_msg);
    }

    #[test]
    #[serial]
    fn test_uses_env_vars_if_environment_is_ci() {
        env::set_var("GITHUB_SHA", "sha_123");
        env::set_var("GITHUB_REF_NAME", "ref_name_123");
        env::set_var("CI", "true");
        fs::remove_file(VERSION_FILE).ok();
        write_version_file().expect("Couldn't run version binary.");
        let file_stream = fs::read(VERSION_FILE).expect("Couldn't read version file");
        let version_file = str::from_utf8(&file_stream).expect("Couldn't read from version file.");
        let error_msg = format!("Got version file contents: \n{}", version_file);
        assert!(version_file.contains("commit: sha_123"), "{}", error_msg);
        assert!(
            version_file.contains("version: ref_name_123"),
            "{}",
            error_msg
        );
        env::remove_var("GITHUB_SHA");
        env::remove_var("GITHUB_REF_NAME");
        env::remove_var("CI");
    }
}
