use reqwest;
use serde::Deserialize;
use serde_json::json;

use super::model::{GetQueryResultsResponse, QueryResponse, ResultSet};

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
            let content: WorkloadIdentityAccessToken =
                r.json().await.expect("Couldn't deserialize.");
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

pub async fn get_bq_results(bq_access_token: &str, query: &str, project: &str) -> ResultSet {
    let client = reqwest::Client::new();
    let api_path = format!(
        "https://www.googleapis.com/bigquery/v2/projects/{}/queries",
        project
    );
    let response = client
        .post(api_path)
        .header("Authorization", format!("Bearer {}", bq_access_token))
        .json(&json!({
            "kind": "bigquery#queryResponse",
            "query": query,
            "useLegacySql": false,
            // TODO - TESTING ONLY
            //"useQueryCache": false,
        }))
        .send()
        .await
        .expect("Failed to get BigQuery query");
    let query_results: GetQueryResultsResponse =
        response.json().await.expect("Couldn't extract body.");
    ResultSet::new(QueryResponse::from(query_results))
}
