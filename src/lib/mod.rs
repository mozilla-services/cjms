pub mod appconfig;
pub mod bigquery;
pub mod cj;
pub mod controllers;
pub mod jobs;
pub mod models;
pub mod settings;
pub mod telemetry;
pub mod version;

#[cfg(test)]
pub mod test_utils {
    use fake::{Fake, StringFaker};

    use crate::settings::Settings;

    pub fn random_ascii_string() -> String {
        const ASCII: &str =
            "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&\'()*+,-./:;<=>?@";
        let f = StringFaker::with(Vec::from(ASCII), 8..90);
        f.fake()
    }

    pub fn random_simple_ascii_string() -> String {
        const ASCII: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ._-";
        let f = StringFaker::with(Vec::from(ASCII), 8..90);
        f.fake()
    }

    pub fn random_currency_or_country() -> String {
        const LETTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let f = StringFaker::with(Vec::from(LETTERS), 1..5);
        f.fake()
    }

    pub fn random_price() -> i32 {
        (99..7899).fake()
    }

    pub fn empty_settings() -> Settings {
        Settings {
            aic_expiration_days: 2,
            authentication: "_".to_string(),
            cj_api_access_token: "_".to_string(),
            cj_cid: "_".to_string(),
            cj_sftp_user: "_".to_string(),
            cj_signature: "_".to_string(),
            cj_subid: "_".to_string(),
            cj_type: "_".to_string(),
            database_url: "_".to_string(),
            environment: "_".to_string(),
            gcp_project: "_".to_string(),
            host: "_".to_string(),
            log_level: "_".to_string(),
            port: 1111,
            sentry_dsn: "_".to_string(),
            sentry_environment: "_".to_string(),
            statsd_host: "_".to_string(),
            statsd_port: 2222,
        }
    }
}
