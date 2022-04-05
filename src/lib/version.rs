use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs;

pub const VERSION_FILE: &str = "version.yaml";

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionInfo {
    pub commit: String,
    pub source: String,
    pub version: String,
}

pub fn read_version(handle: &str) -> VersionInfo {
    let f = fs::File::open(handle).expect("Couldn't read version file.");
    serde_yaml::from_reader(f).expect("Couldn't parse YAML from version file.")
}

pub fn write_version(handle: &str, data: &VersionInfo) {
    let f = fs::File::create(handle).expect("Couldn't create version file.");
    serde_yaml::to_writer(f, &data).expect("Couldn't write YAML to file.");
}

#[cfg(test)]
mod test_version {
    use super::*;
    use serial_test::serial;

    const VERSION_FILE_TEST: &str = "/tmp/version-test.yml";

    #[tokio::test]
    #[serial]
    async fn read_version_success() {
        fs::write(
            VERSION_FILE_TEST,
            "commit: a1b2c3\nsource: source\nversion: version",
        )
        .unwrap();

        let result = read_version(VERSION_FILE_TEST);
        assert_eq!(result.commit, "a1b2c3");
        assert_eq!(result.source, "source");
        assert_eq!(result.version, "version");

        fs::remove_file(VERSION_FILE_TEST).unwrap();
    }

    #[tokio::test]
    #[serial]
    #[should_panic(expected = "Couldn't read version file.")]
    async fn read_version_fails_if_no_file() {
        fs::remove_file(VERSION_FILE_TEST).unwrap();
        read_version(VERSION_FILE_TEST);
    }

    #[tokio::test]
    #[serial]
    #[should_panic(expected = "Couldn't parse YAML from version file.")]
    async fn read_version_fails_if_contents_not_parseable() {
        fs::write(VERSION_FILE_TEST, "not a version file").unwrap();
        read_version(VERSION_FILE_TEST);
        fs::remove_file(VERSION_FILE_TEST).unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn write_version_success() {
        let version_data = VersionInfo {
            commit: "a1b2c3".to_string(),
            source: "source".to_string(),
            version: "version".to_string(),
        };

        write_version(VERSION_FILE_TEST, &version_data);

        let f = fs::read(VERSION_FILE_TEST).unwrap();
        let contents = std::str::from_utf8(&f).unwrap();
        let error_msg = format!("Got version file contents: \n{}", contents);
        assert!(contents.contains("commit: a1b2c3"), "{}", error_msg);
        assert!(contents.contains("source: source"), "{}", error_msg);
        assert!(contents.contains("version: version"), "{}", error_msg);

        fs::remove_file(VERSION_FILE_TEST).unwrap();
    }
}
