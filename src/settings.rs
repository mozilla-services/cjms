use config::{Config, Environment, File, FileFormat};
use std::fs;

#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Settings {
    // Server host and port to run on
    pub host: String,
    pub port: String,
    pub database_url: String,
    // What environment - dev, stage, prod
    pub environment: String,
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
mod test_settings {
    use super::*;
    use serial_test::serial;
    use std::env;
    use std::io::Write;
    use tempfile::NamedTempFile;

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
        env::set_var("HOST", "111.2.3.6");
        env::set_var("PORT", "2222");
        env::set_var(
            "DATABASE_URL",
            "postgres://user:password@127.0.0.1:5432/test",
        );
        env::set_var("ENVIRONMENT", "test");
        let mut mock = MockHasFile::new();
        mock.expect_file().return_const(String::new());
        let actual = _get_settings(mock);
        let expected = Settings {
            host: "111.2.3.6".to_string(),
            port: "2222".to_string(),
            database_url: "postgres://user:password@127.0.0.1:5432/test".to_string(),
            environment: "test".to_string(),
        };
        assert_eq!(expected, actual);
        env::remove_var("HOST");
        env::remove_var("PORT");
        env::remove_var("DATABASE_URL");
        env::remove_var("ENVIRONMENT");
    }

    #[test]
    fn passing_a_file_and_server_address() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "host: 127.1.2.3").unwrap();
        writeln!(file, "port: 2222").unwrap();
        writeln!(file, "database_url: postgres....").unwrap();
        writeln!(file, "environment: prod").unwrap();
        let path = file.into_temp_path();
        let path_str = format!("{}", path.display());
        let mut mock = MockHasFile::new();
        mock.expect_file().return_const(path_str);
        let settings = _get_settings(mock);
        let expected = Settings {
            host: "127.1.2.3".to_string(),
            port: "2222".to_string(),
            database_url: "postgres....".to_string(),
            environment: "prod".to_string(),
        };
        assert_eq!(expected, settings);
        assert_eq!("127.1.2.3:2222", settings.server_address());
    }
}
