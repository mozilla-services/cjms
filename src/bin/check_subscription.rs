use std::env;

use cjms::bigquery::client::{AccessTokenFromEnv, AccessTokenFromMetadata, BQClient};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut use_env = false;
    if args.len() == 2 {
        assert_eq!(
            &args[1],
            "env",
            "Run command with access_type specified `./check_subscription env` or `./check_subscription` to use pod metadata.");
        use_env = true;
    }
    let project = "cjms_nonprod";
    let _ = match use_env {
        true => BQClient::new(project, AccessTokenFromEnv {}, None).await,
        false => BQClient::new(project, AccessTokenFromMetadata {}, None).await,
    };
    Ok(())
}
