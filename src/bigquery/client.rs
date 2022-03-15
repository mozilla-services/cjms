use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use time::OffsetDateTime;

use super::model::{GetQueryResultsResponse, QueryResponse, ResultSet};

pub struct BQClient {
    query_api_url: String,
    access_token: String,
    client: reqwest::Client,
}

impl BQClient {
    pub async fn new(project: &str, token: impl GetAccessToken, domain: Option<&str>) -> BQClient {
        let domain = domain.unwrap_or("https://www.googleapis.com");
        BQClient {
            query_api_url: format!("{}/bigquery/v2/projects/{}/queries", domain, project),
            access_token: token.get().await,
            client: reqwest::Client::new(),
        }
    }
    pub async fn get_bq_results(&self, query: &str) -> ResultSet {
        let resp = self
            .client
            .post(self.query_api_url.as_str())
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&json!({
                "kind": "bigquery#queryResponse",
                "query": query,
                "useLegacySql": false,
            }))
            .send()
            .await
            .expect("Did not successfully query bigquery");
        if resp.status() != 200 {
            panic!("Did not successfully query bigquery. {:?}", resp)
        }
        let query_results: GetQueryResultsResponse =
            resp.json().await.expect("Couldn't extract body.");
        ResultSet::new(QueryResponse::from(query_results))
    }
}

pub fn get_offsetdatetime_from_resultset_column(
    rs: &ResultSet,
    column_name: &str,
) -> OffsetDateTime {
    // Note this panics if we get bad or no data back from big query
    // TODO - Think of cleaner handling strategy (can this wait to see if we actually get bad data in practice?)
    OffsetDateTime::from_unix_timestamp(rs.get_i64_by_name(column_name).unwrap().unwrap())
}

pub fn get_commaseperatedstring_from_resultset_column(rs: &ResultSet, column_name: &str) -> String {
    let promotion_codes_raw = rs
        .get_json_value_by_name(column_name)
        .unwrap()
        .unwrap()
        .to_string();
    let promotion_codes: Vec<String> = serde_json::from_str(&promotion_codes_raw).unwrap();
    promotion_codes.join(",")
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait GetAccessToken {
    async fn get(&self) -> String;
}

#[derive(Deserialize, Debug)]
struct WorkloadIdentityAccessToken {
    pub access_token: String,
    _expires_in: i32,
    _token_type: String,
}

pub struct AccessTokenFromMetadata {}
#[async_trait]
impl GetAccessToken for AccessTokenFromMetadata {
    async fn get(&self) -> String {
        let client = reqwest::Client::new();
        let resp = client
            .get("http://metadata/computeMetadata/v1/instance/service-accounts/default/token")
            .header("Metadata-Flavor", "Google")
            .send()
            .await;
        match resp {
            Ok(r) => {
                let content: WorkloadIdentityAccessToken = r
                    .json()
                    .await
                    .expect("Couldn't deserialize metadata for pod.");
                content.access_token
            }
            Err(e) => {
                panic!("Couldn't get metadata for pod. {:?}", e);
            }
        }
    }
}

pub struct AccessTokenFromEnv {}
#[async_trait]
impl GetAccessToken for AccessTokenFromEnv {
    async fn get(&self) -> String {
        std::env::var("BQ_ACCESS_TOKEN").expect("BQ_ACCESS_TOKEN not found in env.")
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use super::*;
    use crate::utils::random_simple_ascii_string;
    use serde_json::Value;
    use serial_test::serial;
    use wiremock::{
        matchers::{any, body_json, header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    fn fixture_bigquery_response() -> Value {
        let mut file = File::open("tests/fixtures/bigquery_response.json").unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        serde_json::from_str(&data).expect("JSON was not well-formatted")
    }

    #[tokio::test]
    async fn new_should_set_default_big_query_endpoint() {
        let mut mock_token = MockGetAccessToken::new();
        mock_token
            .expect_get()
            .returning(random_simple_ascii_string);
        let project = random_simple_ascii_string();
        let bq = BQClient::new(&project, mock_token, None).await;
        assert_eq!(
            bq.query_api_url,
            format!(
                "https://www.googleapis.com/bigquery/v2/projects/{}/queries",
                project
            )
        );
    }

    #[tokio::test]
    async fn new_should_set_default_passed_in_domain() {
        let mut mock_token = MockGetAccessToken::new();
        mock_token
            .expect_get()
            .returning(random_simple_ascii_string);
        let bq = BQClient::new("its_a_project", mock_token, Some("http://localhost")).await;
        assert_eq!(
            bq.query_api_url,
            "http://localhost/bigquery/v2/projects/its_a_project/queries"
        );
    }

    #[tokio::test]
    async fn new_should_call_get_from_token() {
        let access_token = "called_get_on_token";
        let mut mock_token = MockGetAccessToken::new();
        mock_token
            .expect_get()
            .returning(|| access_token.to_string());
        let bq = BQClient::new(&random_simple_ascii_string(), mock_token, None).await;
        assert_eq!(bq.access_token, access_token);
    }

    #[tokio::test]
    #[serial]
    #[should_panic(expected = "BQ_ACCESS_TOKEN not found in env.")]
    async fn missing_env_var_panics() {
        std::env::remove_var("BQ_ACCESS_TOKEN");
        let token_from_env = AccessTokenFromEnv {};
        BQClient::new(&random_simple_ascii_string(), token_from_env, None).await;
    }

    #[tokio::test]
    #[should_panic(expected = "Couldn't get metadata for pod.")]
    async fn pod_metadata_panics() {
        // As we can't simulate a pod, we test the panic.
        let token_from_metadata = AccessTokenFromMetadata {};
        BQClient::new(&random_simple_ascii_string(), token_from_metadata, None).await;
    }

    #[tokio::test]
    #[serial]
    async fn env_var_sets() {
        let access_token = "env_access_token";
        std::env::set_var("BQ_ACCESS_TOKEN", access_token);
        let token_from_env = AccessTokenFromEnv {};
        let bq = BQClient::new(&random_simple_ascii_string(), token_from_env, None).await;
        assert_eq!(bq.access_token, access_token);
        std::env::remove_var("BQ_ACCESS_TOKEN");
    }

    #[tokio::test]
    async fn bq_client_query_calls_query_endpoint_with_path_headers_and_query() {
        let access_token = "bearer_token_for_request";
        let mut mock_token = MockGetAccessToken::new();
        mock_token
            .expect_get()
            .returning(|| access_token.to_string());
        let mock_google = MockServer::start().await;
        let bq = BQClient::new(
            &random_simple_ascii_string(),
            mock_token,
            Some(&mock_google.uri()),
        )
        .await;
        let expected_path = bq.query_api_url.trim_start_matches(&mock_google.uri());
        let query = r#"SELECT * FROM `dataset.table`;"#;
        let response = ResponseTemplate::new(200).set_body_json(fixture_bigquery_response());
        Mock::given(method("POST"))
            .and(path(expected_path))
            .and(header(
                "Authorization",
                format!("Bearer {}", access_token).as_str(),
            ))
            .and(body_json(&json!({
                "kind": "bigquery#queryResponse",
                "query": query,
                "useLegacySql": false,
            })))
            .respond_with(response)
            .expect(1)
            .mount(&mock_google)
            .await;

        bq.get_bq_results(query).await;
    }

    #[tokio::test]
    #[should_panic(expected = "Did not successfully query bigquery.")]
    async fn bq_client_query_panics_on_500() {
        // This tests the manual panic, not the expect.
        // Not sure how to generate a redirect loop to test the first.
        // This is fine.
        let mut mock_token = MockGetAccessToken::new();
        mock_token
            .expect_get()
            .returning(random_simple_ascii_string);
        let mock_google = MockServer::start().await;
        let bq = BQClient::new(
            &random_simple_ascii_string(),
            mock_token,
            Some(&mock_google.uri()),
        )
        .await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_google)
            .await;
        bq.get_bq_results("").await;
    }

    #[tokio::test]
    #[should_panic(expected = "Couldn't extract body.")]
    async fn bq_client_query_panics_on_bad_body() {
        // This tests the manual panic, not the expect.
        // Not sure how to generate a redirect loop to test the first.
        // This is fine.
        let mut mock_token = MockGetAccessToken::new();
        mock_token
            .expect_get()
            .returning(random_simple_ascii_string);
        let mock_google = MockServer::start().await;
        let bq = BQClient::new(
            &random_simple_ascii_string(),
            mock_token,
            Some(&mock_google.uri()),
        )
        .await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_google)
            .await;
        bq.get_bq_results("").await;
    }

    #[tokio::test]
    async fn bq_client_returns_a_result_set_that_we_can_read_values_from() {
        let mut mock_token = MockGetAccessToken::new();
        mock_token
            .expect_get()
            .returning(random_simple_ascii_string);
        let mock_google = MockServer::start().await;
        let bq = BQClient::new(
            &random_simple_ascii_string(),
            mock_token,
            Some(&mock_google.uri()),
        )
        .await;

        let response = ResponseTemplate::new(200).set_body_json(fixture_bigquery_response());
        Mock::given(any())
            .respond_with(response)
            .mount(&mock_google)
            .await;

        let mut rs = bq.get_bq_results("SELECT * FROM `dataset.table`;").await;
        assert_eq!(rs.row_count(), 3);
        struct TestItem {
            start_date: OffsetDateTime,
            plan_id: String,
            plan_amount: i64,
            promotion_codes: String,
        }
        let mut rows: Vec<TestItem> = Vec::new();
        while rs.next_row() {
            let start_date = get_offsetdatetime_from_resultset_column(&rs, "start_date");
            let promotion_codes =
                get_commaseperatedstring_from_resultset_column(&rs, "promotion_codes");
            rows.push(TestItem {
                start_date,
                plan_id: rs.get_string_by_name("plan_id").unwrap().unwrap(),
                plan_amount: rs.get_i64_by_name("plan_amount").unwrap().unwrap(),
                promotion_codes,
            });
        }
        // One test for each data type. Plus one test that rows are different.
        assert_eq!(
            rows[0].start_date.unix_timestamp(),
            OffsetDateTime::parse("2022-03-10 23:18:49 +0000", "%F %T %z")
                .unwrap()
                .unix_timestamp()
        );
        assert_eq!(rows[0].plan_id, "price_1Iw85dJNcmPzuWtRyhMDdtM7");
        assert_eq!(rows[0].promotion_codes, "a,b");
        assert_eq!(rows[0].plan_amount, 3988);
        assert_eq!(rows[1].plan_amount, 4988);
        assert_eq!(rows[2].plan_amount, 5988);
    }
}
