/*
 * A collection of functions used by bin/check_subscription.rs
 */

use std::env;

use crate::bigquery::client::{AccessTokenFromEnv, AccessTokenFromMetadata, BQClient};
use crate::settings::Settings;

pub async fn get_bqclient(settings: &Settings) -> BQClient {
    // Note we don't have tests that check:
    // - the correct setting of token when using metadata
    // - the correct setting of project when using metadata
    // Take appropriate caution when updating this function.
    match use_env(env::args().collect()) {
        true => BQClient::new(&settings.gcp_project, AccessTokenFromEnv {}, None).await,
        false => BQClient::new(&settings.gcp_project, AccessTokenFromMetadata {}, None).await,
    }
}

fn use_env(args: Vec<String>) -> bool {
    println!("ARGS ARE: {:?}", args);
    let mut use_env = false;
    if args.len() == 2 {
        assert_eq!(
            &args[1],
            "env",
            "Invalid param passed to check_subscription. Use `./check_subscription env` or `./check_subscription` (to use pod metadata).");
        use_env = true;
    }
    use_env
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::settings::test_settings::get_test_settings;

    #[test]
    fn test_use_env_true() {
        let args = vec!["_".to_string(), "env".to_string()];
        let use_env = use_env(args);
        assert!(use_env);
    }

    #[test]
    fn test_use_env_false() {
        let args = vec!["_".to_string()];
        let use_env = use_env(args);
        assert!(!use_env);
    }

    #[test]
    #[should_panic(expected = "Invalid param passed to check_subscription.")]
    fn test_use_env_invalid() {
        let args = vec!["_".to_string(), "_".to_string()];
        let use_env = use_env(args);
        assert!(!use_env);
    }

    #[tokio::test]
    async fn test_get_bq_client_from_env_uses_project_correctly() {
        let settings = get_test_settings("test_gcp_project");
        let bq = get_bqclient(&settings).await;
        assert_eq!(bq.project, "test_gcp_project");
    }
}
