/*
 * A collection of functions used by bin/check_subscription.rs
 */
use crate::bigquery::client::{AccessTokenFromEnv, AccessTokenFromMetadata, BQClient};
use crate::settings::Settings;

pub async fn get_bqclient(settings: &Settings) -> BQClient {
    // Note we don't have tests that check:
    // - the correct setting of token when using metadata
    // - the correct setting of project when using metadata
    // Take appropriate caution when updating this function.
    match use_env(settings) {
        true => BQClient::new(&settings.gcp_project, AccessTokenFromEnv {}, None).await,
        false => BQClient::new(&settings.gcp_project, AccessTokenFromMetadata {}, None).await,
    }
}

fn use_env(settings: &Settings) -> bool {
    match settings.environment.as_str() {
        "dev" | "stage" | "prod" => false,
        "local" | "test" => true,
        _ => panic!("Invalid environment value. Must be local | test | dev | stage | prod."),
    }
}

#[cfg(test)]
mod tests {

    use std::env;

    use super::*;
    use serial_test::serial;

    use crate::settings::test_settings::get_test_settings;
    use crate::test_utils::empty_settings;

    #[test]
    fn test_use_env_true() {
        let mut settings = empty_settings();
        for test_case in ["local", "test"] {
            settings.environment = test_case.to_string();
            let use_env = use_env(&settings);
            assert!(use_env);
        }
    }

    #[test]
    fn test_use_env_false() {
        let mut settings = empty_settings();
        for test_case in ["dev", "stage", "prod"] {
            settings.environment = test_case.to_string();
            let use_env = use_env(&settings);
            assert!(!use_env);
        }
    }

    #[test]
    #[should_panic(expected = "Invalid environment value. Must be local | test |")]
    fn test_use_env_invalid() {
        let mut settings = empty_settings();
        settings.environment = "misc".to_string();
        use_env(&settings);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_bq_client_from_env_uses_project_correctly() {
        env::set_var("BQ_ACCESS_TOKEN", "a token");
        let mut settings = get_test_settings("test_gcp_project");
        settings.environment = "local".to_string();
        let bq = get_bqclient(&settings).await;
        assert_eq!(bq.project, "test_gcp_project");
        env::remove_var("BQ_ACCESS_TOKEN");
    }
}
