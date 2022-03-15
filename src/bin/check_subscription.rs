use std::env;

use cjms::bigquery::client::{AccessTokenFromEnv, AccessTokenFromMetadata, BQClient};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    // TODO Replace with environment variable when available
    let project = "cjms_nonprod";
    let _ = match use_env(args) {
        true => BQClient::new(project, AccessTokenFromEnv {}, None).await,
        false => BQClient::new(project, AccessTokenFromMetadata {}, None).await,
    };
    Ok(())
}

fn use_env(args: Vec<String>) -> bool {
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
mod test_bin_check_subscription {
    use super::*;

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
}
