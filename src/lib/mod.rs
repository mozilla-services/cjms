pub mod appconfig;
pub mod bigquery;
pub mod check_subscriptions;
pub mod cj;
pub mod controllers;
pub mod models;
pub mod settings;

#[cfg(test)]
pub mod test_utils {
    use fake::{Fake, StringFaker};

    use crate::settings::Settings;

    pub fn random_simple_ascii_string() -> String {
        const ASCII: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ._-";
        let f = StringFaker::with(Vec::from(ASCII), 8..90);
        f.fake()
    }

    pub fn empty_settings() -> Settings {
        Settings {
            host: "_".to_string(),
            port: "_".to_string(),
            database_url: "_".to_string(),
            environment: "_".to_string(),
            gcp_project: "_".to_string(),
        }
    }
}