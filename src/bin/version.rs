use lib::version::{write_version, VersionInfo, VERSION_FILE};
use std::env;
use std::process::Command;
use std::str;

// Note that this binary is run as part of the Docker image build process.
// Therefore cannot initialize telemetry tools like tracing and Sentry here.
fn main() -> std::io::Result<()> {
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

    let version_info = VersionInfo {
        source: env!("CARGO_PKG_REPOSITORY").to_string(),
        commit: sha,
        version: tag,
    };
    write_version(VERSION_FILE, &version_info);

    Ok(())
}

#[cfg(test)]
mod test_bin_version {
    use super::*;
    use serial_test::serial;
    use std::fs;

    #[test]
    #[serial]
    fn test_writes_a_new_file_that_has_source() {
        fs::remove_file(VERSION_FILE).ok();
        main().expect("Couldn't run version binary.");
        let file_stream = fs::read(VERSION_FILE).expect("Couldn't read version file");
        let version_file = str::from_utf8(&file_stream).expect("Couldn't read from version file.");
        let source = env!("CARGO_PKG_REPOSITORY");
        let error_msg = format!("Got version file contents: \n{}", version_file);
        assert!(
            version_file.contains(format!(r#"source: "{}""#, source).as_str()),
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
        main().expect("Couldn't run version binary.");
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
