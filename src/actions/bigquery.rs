use reqwest::{self, Response};
use serde::{Deserialize, Serialize};
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
        .post("https://www.googleapis.com/bigquery/v2/projects/moz-fx-cjms-nonprod-9a36/queries")
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorProto {
    /// Debugging information. This property is internal to Google and should not be used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_info: Option<String>,
    /// Specifies where the error occurred, if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// A human-readable description of the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// A short error code that summarizes the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobReference {
    /// [Required] The ID of the job. The ID must contain only letters (a-z, A-Z), numbers (0-9), underscores (_), or dashes (-). The maximum length is 1,024 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    /// The geographic location of the job. See details at https://cloud.google.com/bigquery/docs/locations#specifying_your_location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// [Required] The ID of the project containing this job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableCell {
    #[serde(rename = "v", skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRow {
    /// Represents a single row in the result set, consisting of one or more fields.
    #[serde(rename = "f", skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<TableCell>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableFieldSchemaCategories {
    /// A list of category resource names. For example, \"projects/1/taxonomies/2/categories/3\". At most 5 categories are allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<String>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableFieldSchemaPolicyTags {
    /// A list of category resource names. For example, \"projects/1/location/eu/taxonomies/2/policyTags/3\". At most 1 policy tag is allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableFieldSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<TableFieldSchemaCategories>,
    /// [Optional] The field description. The maximum length is 1,024 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// [Optional] Describes the nested schema fields if the type property is set to RECORD.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<TableFieldSchema>>,
    /// [Optional] The field mode. Possible values include NULLABLE, REQUIRED and REPEATED. The default value is NULLABLE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// [Required] The field name. The name must contain only letters (a-z, A-Z), numbers (0-9), or underscores (_), and must start with a letter or underscore. The maximum length is 128 characters.
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_tags: Option<TableFieldSchemaPolicyTags>,
    /// [Required] The field data type. Possible values include STRING, BYTES, INTEGER, INT64 (same as INTEGER), FLOAT, FLOAT64 (same as FLOAT), NUMERIC, BIGNUMERIC, BOOLEAN, BOOL (same as BOOLEAN), TIMESTAMP, DATE, TIME, DATETIME, RECORD (where RECORD indicates that the field contains a nested schema) or STRUCT (same as RECORD).
    pub r#type: FieldType,
}

impl TableFieldSchema {
    pub fn new(field_name: &str, field_type: FieldType) -> Self {
        Self {
            categories: None,
            description: None,
            fields: None,
            mode: None,
            name: field_name.into(),
            policy_tags: None,
            r#type: field_type,
        }
    }

    pub fn integer(field_name: &str) -> Self {
        Self {
            categories: None,
            description: None,
            fields: None,
            mode: None,
            name: field_name.into(),
            policy_tags: None,
            r#type: FieldType::Integer,
        }
    }

    pub fn float(field_name: &str) -> Self {
        Self {
            categories: None,
            description: None,
            fields: None,
            mode: None,
            name: field_name.into(),
            policy_tags: None,
            r#type: FieldType::Float,
        }
    }

    pub fn bool(field_name: &str) -> Self {
        Self {
            categories: None,
            description: None,
            fields: None,
            mode: None,
            name: field_name.into(),
            policy_tags: None,
            r#type: FieldType::Bool,
        }
    }

    pub fn string(field_name: &str) -> Self {
        Self {
            categories: None,
            description: None,
            fields: None,
            mode: None,
            name: field_name.into(),
            policy_tags: None,
            r#type: FieldType::String,
        }
    }

    pub fn record(field_name: &str, fields: Vec<TableFieldSchema>) -> Self {
        Self {
            categories: None,
            description: None,
            fields: Some(fields),
            mode: None,
            name: field_name.into(),
            policy_tags: None,
            r#type: FieldType::Record,
        }
    }

    pub fn bytes(field_name: &str) -> Self {
        Self {
            categories: None,
            description: None,
            fields: None,
            mode: None,
            name: field_name.into(),
            policy_tags: None,
            r#type: FieldType::Bytes,
        }
    }

    pub fn numeric(field_name: &str) -> Self {
        Self {
            categories: None,
            description: None,
            fields: None,
            mode: None,
            name: field_name.into(),
            policy_tags: None,
            r#type: FieldType::Numeric,
        }
    }

    pub fn big_numeric(field_name: &str) -> Self {
        Self {
            categories: None,
            description: None,
            fields: None,
            mode: None,
            name: field_name.into(),
            policy_tags: None,
            r#type: FieldType::Bignumeric,
        }
    }

    pub fn timestamp(field_name: &str) -> Self {
        Self {
            categories: None,
            description: None,
            fields: None,
            mode: None,
            name: field_name.into(),
            policy_tags: None,
            r#type: FieldType::Timestamp,
        }
    }

    pub fn date(field_name: &str) -> Self {
        Self {
            categories: None,
            description: None,
            fields: None,
            mode: None,
            name: field_name.into(),
            policy_tags: None,
            r#type: FieldType::Date,
        }
    }

    pub fn time(field_name: &str) -> Self {
        Self {
            categories: None,
            description: None,
            fields: None,
            mode: None,
            name: field_name.into(),
            policy_tags: None,
            r#type: FieldType::Time,
        }
    }

    pub fn date_time(field_name: &str) -> Self {
        Self {
            categories: None,
            description: None,
            fields: None,
            mode: None,
            name: field_name.into(),
            policy_tags: None,
            r#type: FieldType::Datetime,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FieldType {
    String,
    Bytes,
    Integer,
    Int64, // same as INTEGER
    Float,
    Float64, // same as FLOAT
    Numeric,
    Bignumeric,
    Boolean,
    Bool, // same as BOOLEAN
    Timestamp,
    Date,
    Time,
    Datetime,
    Record, // where RECORD indicates that the field contains a nested schema
    Struct, // same as RECORD
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableSchema {
    /// Describes the fields in a table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<TableFieldSchema>>,
}

impl TableSchema {
    pub fn new(fields: Vec<TableFieldSchema>) -> Self {
        Self {
            fields: Some(fields),
        }
    }

    pub fn fields(&self) -> &Option<Vec<TableFieldSchema>> {
        &self.fields
    }

    pub fn field_count(&self) -> usize {
        self.fields.as_ref().map_or(0, |fields| fields.len())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponse {
    /// Whether the query result was fetched from the query cache.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_hit: Option<bool>,
    /// [Output-only] The first errors or warnings encountered during the running of the job. The final message includes the number of errors that caused the process to stop. Errors here do not necessarily mean that the job has completed or was unsuccessful.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ErrorProto>>,
    /// Whether the query has completed or not. If rows or totalRows are present, this will always be true. If this is false, totalRows will not be available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_complete: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_reference: Option<JobReference>,
    /// The resource type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// [Output-only] The number of rows affected by a DML statement. Present only for DML statements INSERT, UPDATE or DELETE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_dml_affected_rows: Option<String>,
    /// A token used for paging results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
    /// An object with as many results as can be contained within the maximum permitted reply size. To get any additional rows, you can call GetQueryResults and specify the jobReference returned above.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<TableRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<TableSchema>,
    /// The total number of bytes processed for this query. If this query was a dry run, this is the number of bytes that would be processed if the query were run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bytes_processed: Option<String>,
    /// The total number of rows in the complete query result set, which can be more than the number of rows in this single page of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_rows: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetQueryResultsResponse {
    /// Whether the query result was fetched from the query cache.
    pub cache_hit: Option<bool>,
    /// [Output-only] The first errors or warnings encountered during the running of the job. The final message includes the number of errors that caused the process to stop. Errors here do not necessarily mean that the job has completed or was unsuccessful.
    pub errors: Option<Vec<ErrorProto>>,
    /// A hash of this response.
    pub etag: Option<String>,
    /// Whether the query has completed or not. If rows or totalRows are present, this will always be true. If this is false, totalRows will not be available.
    pub job_complete: Option<bool>,
    /// Reference to the BigQuery Job that was created to run the query. This field will be present even if the original request timed out, in which case GetQueryResults can be used to read the results once the query has completed. Since this API only returns the first page of results, subsequent pages can be fetched via the same mechanism (GetQueryResults).
    pub job_reference: Option<JobReference>,
    /// The resource type of the response.
    pub kind: Option<String>,
    /// [Output-only] The number of rows affected by a DML statement. Present only for DML statements INSERT, UPDATE or DELETE.
    pub num_dml_affected_rows: Option<String>,
    /// A token used for paging results.
    pub page_token: Option<String>,
    /// An object with as many results as can be contained within the maximum permitted reply size. To get any additional rows, you can call GetQueryResults and specify the jobReference returned above. Present only when the query completes successfully.
    pub rows: Option<Vec<TableRow>>,
    /// The schema of the results. Present only when the query completes successfully.
    pub schema: Option<TableSchema>,
    /// The total number of bytes processed for this query.
    pub total_bytes_processed: Option<String>,
    /// The total number of rows in the complete query result set, which can be more than the number of rows in this single page of results. Present only when the query completes successfully.
    pub total_rows: Option<String>,
}

impl From<GetQueryResultsResponse> for QueryResponse {
    fn from(resp: GetQueryResultsResponse) -> Self {
        Self {
            cache_hit: resp.cache_hit,
            errors: resp.errors,
            job_complete: resp.job_complete,
            job_reference: resp.job_reference,
            kind: resp.kind,
            num_dml_affected_rows: resp.num_dml_affected_rows,
            page_token: resp.page_token,
            rows: resp.rows,
            schema: resp.schema,
            total_bytes_processed: resp.total_bytes_processed,
            total_rows: resp.total_rows,
        }
    }
}
