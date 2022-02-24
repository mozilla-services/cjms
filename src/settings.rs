use config::{Config, Environment, File, FileFormat};

#[derive(serde::Deserialize, PartialEq, Eq, Debug)]
pub struct Settings {
    pub host: String,
    pub port: String,
}

impl Settings {
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

pub fn get_settings(config_file: Option<&String>) -> Settings {
    let mut builder = Config::builder();
    // Either we use a config file, or we use environment variables
    builder = match config_file {
        Some(filename) => builder.add_source(File::new(filename, FileFormat::Yaml)),
        None => builder.add_source(Environment::default()),
    };
    let config = builder.build().expect("Config couldn't be built.");
    match config.try_deserialize::<Settings>() {
        Ok(settings) => {
            println!("settings are: {:?}", settings);
            settings
        }
        Err(e) => panic!("Aborting. Config didn't match serialization. {:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use std::io::{Write};
    use tempfile::NamedTempFile;

    #[test]
    #[serial]
    #[should_panic]
    fn test_get_settings_missing_envvars() {
        // If you have set environment variables, this test may fail
        let _ = get_settings(None);
    }

    #[test]
    #[serial]
    #[should_panic]
    fn test_get_settings_missing_file() {
        // If you have set environment variables, this test may fail
        let _ = get_settings(Some(&String::from("this_file_doesnt_exist.txt")));
    }

    #[test]
    #[serial]
    fn test_get_settings_with_envvars() {
        env::set_var("HOST", "111.2.3.6");
        env::set_var("PORT", "2222");
        let actual = get_settings(None);
        let expected = Settings {
            host: "111.2.3.6".to_string(),
            port: "2222".to_string(),
        };
        assert_eq!(expected, actual);
        env::remove_var("HOST");
        env::remove_var("PORT");
    }

    #[test]
    #[serial]
    fn test_server_address() {
        env::set_var("HOST", "111.2.3.5");
        env::set_var("PORT", "2222");
        let settings = get_settings(None);
        let actual = settings.server_address();
        assert_eq!("111.2.3.5:2222", actual);
        env::remove_var("HOST");
        env::remove_var("PORT");
    }

    #[test]
    fn test_passing_a_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "host: 127.0.0.1").unwrap();
        writeln!(file, "port: 2222").unwrap();
        let path = file.into_temp_path();
        let path_str = format!("{}", path.display());
        let actual = get_settings(Some(&path_str));
        let expected = Settings {
            host: "127.0.0.1".to_string(),
            port: "2222".to_string(),
        };
        assert_eq!(expected, actual);

    }
}
