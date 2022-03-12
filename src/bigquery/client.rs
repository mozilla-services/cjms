use async_trait::async_trait;
use serde::Deserialize;

struct BQClient {
    query_api_url: String,
    access_token: String,
}

impl BQClient {
    pub async fn new(project: &str, token: impl GetAccessToken, domain: Option<&str>) -> BQClient {
        let domain = domain.unwrap_or("https://www.googleapis.com");
        BQClient {
            query_api_url: format!("{}/bigquery/v2/projects/{}/queries", domain, project),
            access_token: token.get().await,
        }
    }
    pub async fn get_bq_results(&self, query: &str) {
        println!("{}", query);
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
trait GetAccessToken {
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
    use super::*;
    use crate::test_utils::random_ascii_string;
    use serial_test::serial;
    use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn new_should_set_default_big_query_endpoint() {
        let mut mock = MockGetAccessToken::new();
        mock.expect_get().times(1).returning(random_ascii_string);
        let project = random_ascii_string();
        let bq = BQClient::new(&project, mock, None).await;
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
        let mut mock = MockGetAccessToken::new();
        mock.expect_get().times(1).returning(random_ascii_string);
        let bq = BQClient::new("its_a_project", mock, Some("http://localhost")).await;
        assert_eq!(
            bq.query_api_url,
            "http://localhost/bigquery/v2/projects/its_a_project/queries"
        );
    }

    #[tokio::test]
    async fn new_should_call_get_from_token() {
        let access_token = "called_get_on_token";
        let mut mock = MockGetAccessToken::new();
        mock.expect_get()
            .times(1)
            .returning(|| access_token.to_string());
        let bq = BQClient::new(&random_ascii_string(), mock, None).await;
        assert_eq!(bq.access_token, access_token);
    }

    #[tokio::test]
    #[serial]
    #[should_panic(expected = "BQ_ACCESS_TOKEN not found in env.")]
    async fn missing_env_var_panics() {
        std::env::remove_var("BQ_ACCESS_TOKEN");
        let token_from_env = AccessTokenFromEnv {};
        BQClient::new(&random_ascii_string(), token_from_env, None).await;
    }

    #[tokio::test]
    #[should_panic(expected = "Couldn't get metadata for pod.")]
    async fn pod_metadata_panics() {
        // As we can't simulate a pod, we test the panic.
        let token_from_metadata = AccessTokenFromMetadata {};
        BQClient::new(&random_ascii_string(), token_from_metadata, None).await;
    }

    #[tokio::test]
    #[serial]
    async fn env_var_sets() {
        let access_token = "env_access_token";
        std::env::set_var("BQ_ACCESS_TOKEN", access_token);
        let token_from_env = AccessTokenFromEnv {};
        let bq = BQClient::new(&random_ascii_string(), token_from_env, None).await;
        assert_eq!(bq.access_token, access_token);
        std::env::remove_var("BQ_ACCESS_TOKEN");
    }

    #[tokio::test]
    async fn bq_client_query_calls_query_endpoint_with_path_headers_and_query() {
        let access_token = "called_get_on_token";
        let mut mock_access_token = MockGetAccessToken::new();
        mock_access_token
            .expect_get()
            .times(1)
            .returning(|| access_token.to_string());

        let mock_google = MockServer::start().await;

        /*
        Mock::given(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_google)
            .await;
        */
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_google)
            .await;

        let bq = BQClient::new(
            &random_ascii_string(),
            mock_access_token,
            Some(&mock_google.uri()),
        )
        .await;

        let query = r#"SELECT * FROM `dataset.table`;"#;
        bq.get_bq_results(query).await;
    }
}
