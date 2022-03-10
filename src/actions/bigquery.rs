use reqwest::{self, Response};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Debug)]
pub struct WorkloadIdentityAccessToken {
    pub access_token: String,
    pub expires_in: i32,
    pub token_type: String,
}

pub async fn get_access_token_from_metadata() -> String {
    let client = reqwest::Client::new();
    let resp = client
        .get("http://metadata/computeMetadata/v1/instance/service-accounts/default/token")
        .header("Metadata-Flavor", "Google")
        .send()
        .await;
    match resp {
        Ok(r) => {
            println!("The response is: {:?}", r);
            let content: WorkloadIdentityAccessToken =
                r.json().await.expect("Couldn't deserialize.");
            println!("The json is: {:?}", content);
            content.access_token
        }
        Err(e) => {
            println!("The error is: {:?}", e);
            panic!("We can't go on.");
        }
    }
}

pub async fn get_access_token_from_env() -> String {
    std::env::var("BQ_ACCESS_TOKEN").expect("BQ_ACCESS_TOKEN not available")
}

pub async fn run_bq_table_get(bq_access_token: String, query: &str) -> Response {
    let client = reqwest::Client::new();
    client
        .post("https://www.googleapis.com/bigquery/v2/projects/mozdata/queries")
        .header("Authorization", format!("Bearer {}", bq_access_token))
        .json(&json!({
            "kind": "ARRAY",
            "query": query,
            "useLegacySql": false
        }))
        .send()
        .await
        .expect("Failed to get BigQuery query")
}
