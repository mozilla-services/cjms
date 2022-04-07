use config::{Config, Environment, File, FileFormat};
use std::fs;

#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Settings {
    pub aic_expiration_days: u64,
    pub authentication: String,
    pub cj_cid: String,
    pub cj_sftp_user: String,
    pub cj_signature: String,
    pub cj_subid: String,
    pub cj_type: String,
    pub database_url: String,
    pub environment: String,
    pub gcp_project: String,
    pub host: String,
    pub log_level: String,
    pub port: u16,
    pub sentry_dsn: String,
    pub sentry_environment: String,
    pub statsd_host: String,
    pub statsd_port: u16,
}

impl Settings {
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg_attr(test, mockall::automock)]
pub trait HasFile {
    fn file(&self) -> &str;
}
pub struct SettingsFile {}
impl HasFile for SettingsFile {
    fn file(&self) -> &str {
        "settings.yaml"
    }
}

fn _get_settings(settings: impl HasFile) -> Settings {
    let mut builder = Config::builder();
    // Either we use a settings.yaml file, or environment variables
    let settings_file = settings.file();
    builder = match fs::metadata(settings_file) {
        Ok(metadata) => match metadata.is_file() {
            true => builder.add_source(File::new(settings_file, FileFormat::Yaml)),
            false => panic!("Given settings file is not a file"),
        },
        Err(error) => match error.kind() {
            std::io::ErrorKind::NotFound => builder.add_source(Environment::default()),
            _ => panic!("Unexpected error when loading metadata."),
        },
    };
    let config = builder.build().expect("Config couldn't be built.");
    match config.try_deserialize::<Settings>() {
        Ok(settings) => settings,
        Err(e) => panic!("Config didn't match serialization. {:?}", e),
    }
}

pub fn get_settings() -> Settings {
    _get_settings(SettingsFile {})
}

#[cfg(test)]
pub mod test_settings {
    use super::*;
    use pretty_assertions::assert_eq;
    use serial_test::serial;
    use std::env;
    use std::io::Write;
    use tempfile::NamedTempFile;

    pub fn get_test_settings(gcp_project: &str) -> Settings {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "aic_expiration_days: 22222").unwrap();
        writeln!(file, "authentication: auth a pass").unwrap();
        writeln!(file, "cj_cid: cid").unwrap();
        writeln!(file, "cj_sftp_user: sftp_user").unwrap();
        writeln!(file, "cj_signature: signature").unwrap();
        writeln!(file, "cj_subid: subid").unwrap();
        writeln!(file, "cj_type: type").unwrap();
        writeln!(file, "database_url: postgres....").unwrap();
        writeln!(file, "environment: prod").unwrap();
        writeln!(file, "gcp_project: {}", gcp_project).unwrap();
        writeln!(file, "host: 127.1.2.3").unwrap();
        writeln!(file, "log_level: info").unwrap();
        writeln!(file, "port: 2222").unwrap();
        writeln!(file, "sentry_dsn: somevalue").unwrap();
        writeln!(file, "sentry_environment: somevalue").unwrap();
        writeln!(file, "statsd_host: 0.0.0.0").unwrap();
        writeln!(file, "statsd_port: 10101").unwrap();
        let path = file.into_temp_path();
        let path_str = format!("{}", path.display());
        let mut mock = MockHasFile::new();
        mock.expect_file().return_const(path_str);
        _get_settings(mock)
    }

    // Existing environment variables may mess with these tests

    #[test]
    #[serial]
    #[should_panic(expected = "Config didn't match serialization.")]
    fn missing_settings_values_panics() {
        env::set_var("HOST", "111.2.3.6");
        let mut mock = MockHasFile::new();
        mock.expect_file()
            .return_const("gobbledygook.yaml".to_string());
        let _ = _get_settings(mock);
        env::remove_var("HOST");
    }

    #[test]
    #[serial]
    #[should_panic(expected = "Given settings file is not a file")]
    fn passing_settings_file_that_is_a_directory_panics() {
        let mut mock = MockHasFile::new();
        mock.expect_file().return_const("src".to_string());
        let _ = _get_settings(mock);
    }

    #[test]
    #[serial]
    #[should_panic(expected = "Config couldn't be built.")]
    fn settings_file_is_not_yaml() {
        let mut mock = MockHasFile::new();
        mock.expect_file().return_const("README.md".to_string());
        let _ = _get_settings(mock);
    }

    #[test]
    #[serial]
    fn get_settings_with_envvars() {
        env::set_var("AIC_EXPIRATION_DAYS", "121212");
        env::set_var("AUTHENTICATION", "auth pass");
        env::set_var("CJ_CID", "test cj cid");
        env::set_var("CJ_SFTP_USER", "test cj sftp user");
        env::set_var("CJ_SIGNATURE", "test cj signature");
        env::set_var("CJ_SUBID", "test cj subid");
        env::set_var("CJ_TYPE", "test cj type");
        env::set_var(
            "DATABASE_URL",
            "postgres://user:password@127.0.0.1:5432/test",
        );
        env::set_var("ENVIRONMENT", "test");
        env::set_var("GCP_PROJECT", "a--te-st-pr0j");
        env::set_var("HOST", "111.2.3.6");
        env::set_var("LOG_LEVEL", "info");
        env::set_var("PORT", "2222");
        env::set_var("SENTRY_DSN", "somevalue");
        env::set_var("SENTRY_ENVIRONMENT", "somevalue");
        env::set_var("STATSD_HOST", "0.0.0.0");
        env::set_var("STATSD_PORT", "10101");
        let mut mock = MockHasFile::new();
        mock.expect_file().return_const(String::new());
        let actual = _get_settings(mock);
        let expected = Settings {
            aic_expiration_days: 121212,
            authentication: "auth pass".to_string(),
            cj_cid: "test cj cid".to_string(),
            cj_sftp_user: "test cj sftp user".to_string(),
            cj_signature: "test cj signature".to_string(),
            cj_subid: "test cj subid".to_string(),
            cj_type: "test cj type".to_string(),
            database_url: "postgres://user:password@127.0.0.1:5432/test".to_string(),
            environment: "test".to_string(),
            gcp_project: "a--te-st-pr0j".to_string(),
            host: "111.2.3.6".to_string(),
            log_level: "info".to_string(),
            port: 2222,
            sentry_dsn: "somevalue".to_string(),
            sentry_environment: "somevalue".to_string(),
            statsd_host: "0.0.0.0".to_string(),
            statsd_port: 10101,
        };
        assert_eq!(expected, actual);
        env::remove_var("AIC_EXPIRATION_DAYS");
        env::remove_var("AUTHENTICATION");
        env::remove_var("CJ_CID");
        env::remove_var("CJ_SFTP_USER");
        env::remove_var("CJ_SIGNATURE");
        env::remove_var("CJ_SUBID");
        env::remove_var("CJ_TYPE");
        env::remove_var("DATABASE_URL");
        env::remove_var("ENVIRONMENT");
        env::remove_var("GCP_PROJECT");
        env::remove_var("HOST");
        env::remove_var("LOG_LEVEL");
        env::remove_var("PORT");
        env::remove_var("SENTRY_DSN");
        env::remove_var("SENTRY_ENVIRONMENT");
        env::remove_var("STATSD_HOST");
        env::remove_var("STATSD_PORT");
    }

    #[test]
    fn passing_a_file_and_server_address() {
        let settings = get_test_settings("a-gcp-Pr0j3ct");
        let expected = Settings {
            aic_expiration_days: 22222,
            authentication: "auth a pass".to_string(),
            cj_cid: "cid".to_string(),
            cj_sftp_user: "sftp_user".to_string(),
            cj_signature: "signature".to_string(),
            cj_subid: "subid".to_string(),
            cj_type: "type".to_string(),
            database_url: "postgres....".to_string(),
            environment: "prod".to_string(),
            gcp_project: "a-gcp-Pr0j3ct".to_string(),
            host: "127.1.2.3".to_string(),
            log_level: "info".to_string(),
            port: 2222,
            sentry_dsn: "somevalue".to_string(),
            sentry_environment: "somevalue".to_string(),
            statsd_host: "0.0.0.0".to_string(),
            statsd_port: 10101,
        };
        assert_eq!(expected, settings);
        assert_eq!("127.1.2.3:2222", settings.server_address());
    }
}
