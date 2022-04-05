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

    pub fn random_simple_ascii_string() -> String {
        const ASCII: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ._-";
        let f = StringFaker::with(Vec::from(ASCII), 8..90);
        f.fake()
    }

    pub fn empty_settings() -> Settings {
        Settings {
            authentication: "_".to_string(),
            cj_cid: "_".to_string(),
            cj_signature: "_".to_string(),
            cj_subid: "_".to_string(),
            cj_type: "_".to_string(),
            database_url: "_".to_string(),
            environment: "_".to_string(),
            gcp_project: "_".to_string(),
            host: "_".to_string(),
            log_level: "_".to_string(),
            port: "_".to_string(),
            sentry_dsn: "_".to_string(),
            sentry_environment: "_".to_string(),
            statsd_host: "_".to_string(),
            statsd_port: "_".to_string(),
        }
    }
}
