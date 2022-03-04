use cjms::controllers::custodial::VERSION_FILE;
use std::fs;
use std::process::Command;
use std::str;

fn main() -> std::io::Result<()> {
    // Repo link
    let source = env!("CARGO_PKG_REPOSITORY");

    // Commit SHA
    let rev_parse_out = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output()
        .expect("Failed to execute git rev-parse");
    let rev_parse =
        str::trim(str::from_utf8(&rev_parse_out.stdout).expect("Failed to read output."));

    // Version
    let tags_out = Command::new("git")
        .arg("describe")
        .arg("--tags")
        .output()
        .expect("Failed to execute git rev-parse");
    let tag = str::trim(str::from_utf8(&tags_out.stdout).expect("Failed to read output."));

    fs::write(
        VERSION_FILE,
        format!(
            "source: {}\ncommit: {}\nversion: {}\n",
            source, rev_parse, tag
        ),
    )
    .expect("Failed to write file.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writes_a_new_file_that_has_source() {
        fs::remove_file(VERSION_FILE).ok();
        main().expect("Couldn't run version binary.");
        let file_stream = fs::read(VERSION_FILE).expect("Couldn't read version file");
        let version_file = str::from_utf8(&file_stream).expect("Couldn't read from version file.");
        let source = env!("CARGO_PKG_REPOSITORY");
        let error_msg = format!("Couldn't find source in: \n{}", version_file);
        assert!(
            version_file.contains(format!("source: {}", source).as_str()),
            "{}",
            error_msg
        );
        assert!(version_file.contains("commit:"), "{}", error_msg);
        assert!(version_file.contains("version:"), "{}", error_msg);
    }
}
