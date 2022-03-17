/*
This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.

AND

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.


The contents of this file were taken from various parts of https://github.com/lquerel/gcp-bigquery-client
and adapted to our needs.
Sincere thanks to https://github.com/lquerel and the gcp-bigquery-client contributors
https://github.com/lquerel/gcp-bigquery-client/graphs/contributors.

*/

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Error, Debug)]
pub enum BQError {
    #[error("BQError: No data available. The result set is positioned before the first or after the last row. Try to call the method next on your result set.")]
    NoDataAvailable,

    #[error("BQError: Invalid column index (col_index: {col_index})")]
    InvalidColumnIndex { col_index: usize },

    #[error("BQError: Invalid column name (col_name: {col_name})")]
    InvalidColumnName { col_name: String },

    #[error("BQError: Invalid column type (col_index: {col_index}, col_type: {col_type}, type_requested: {type_requested})")]
    InvalidColumnType {
        col_index: usize,
        col_type: String,
        type_requested: String,
    },

    #[error("BQError: Could not cast integer from i64 to i32")]
    IntegerCastUnsuccessful,
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

/// Set of rows in response to a SQL query
#[derive(Debug)]
pub struct ResultSet {
    cursor: i64,
    row_count: i64,
    query_response: QueryResponse,
    fields: HashMap<String, usize>,
}

impl ResultSet {
    pub fn new(query_response: QueryResponse) -> Self {
        if query_response.job_complete.unwrap_or(false) {
            // rows and tables schema are only present for successfully completed jobs.
            let row_count = query_response.rows.as_ref().map_or(0, Vec::len) as i64;
            let table_schema = query_response.schema.as_ref().expect("Expecting a schema");
            let table_fields = table_schema
                .fields
                .as_ref()
                .expect("Expecting a non empty list of fields");
            let fields: HashMap<String, usize> = table_fields
                .iter()
                .enumerate()
                .map(|(pos, field)| (field.name.clone(), pos))
                .collect();
            Self {
                cursor: -1,
                row_count,
                query_response,
                fields,
            }
        } else {
            Self {
                cursor: -1,
                row_count: 0,
                query_response,
                fields: HashMap::new(),
            }
        }
    }

    pub fn query_response(&self) -> &QueryResponse {
        &self.query_response
    }

    /// Moves the cursor froward one row from its current position.
    /// A ResultSet cursor is initially positioned before the first row; the first call to the method next makes the
    /// first row the current row; the second call makes the second row the current row, and so on.
    pub fn next_row(&mut self) -> bool {
        if self.cursor == (self.row_count - 1) {
            false
        } else {
            self.cursor += 1;
            true
        }
    }

    /// Total number of rows in this result set.
    pub fn row_count(&self) -> usize {
        self.row_count as usize
    }

    /// List of column names for this result set.
    pub fn column_names(&self) -> Vec<String> {
        self.fields.keys().cloned().collect()
    }

    /// Returns the index for a column name.
    pub fn column_index(&self, column_name: &str) -> Option<&usize> {
        self.fields.get(column_name)
    }

    pub fn get_i64(&self, col_index: usize) -> Result<Option<i64>, BQError> {
        let json_value = self.get_json_value(col_index)?;
        match &json_value {
            None => Ok(None),
            Some(json_value) => match json_value {
                serde_json::Value::Number(value) => Ok(value.as_i64()),
                serde_json::Value::String(value) => {
                    match (value.parse::<i64>(), value.parse::<f64>()) {
                        (Ok(v), _) => Ok(Some(v)),
                        (Err(_), Ok(v)) => Ok(Some(v as i64)),
                        _ => Err(BQError::InvalidColumnType {
                            col_index,
                            col_type: ResultSet::json_type(json_value),
                            type_requested: "I64".into(),
                        }),
                    }
                }
                _ => Err(BQError::InvalidColumnType {
                    col_index,
                    col_type: ResultSet::json_type(json_value),
                    type_requested: "I64".into(),
                }),
            },
        }
    }

    pub fn get_i64_by_name(&self, col_name: &str) -> Result<Option<i64>, BQError> {
        let col_index = self.fields.get(col_name);
        match col_index {
            None => Err(BQError::InvalidColumnName {
                col_name: col_name.into(),
            }),
            Some(col_index) => self.get_i64(*col_index),
        }
    }

    pub fn get_f64(&self, col_index: usize) -> Result<Option<f64>, BQError> {
        let json_value = self.get_json_value(col_index)?;
        match &json_value {
            None => Ok(None),
            Some(json_value) => match json_value {
                serde_json::Value::Number(value) => Ok(value.as_f64()),
                serde_json::Value::String(value) => {
                    let value: Result<f64, _> = value.parse();
                    match &value {
                        Err(_) => Err(BQError::InvalidColumnType {
                            col_index,
                            col_type: ResultSet::json_type(json_value),
                            type_requested: "F64".into(),
                        }),
                        Ok(value) => Ok(Some(*value)),
                    }
                }
                _ => Err(BQError::InvalidColumnType {
                    col_index,
                    col_type: ResultSet::json_type(json_value),
                    type_requested: "F64".into(),
                }),
            },
        }
    }

    pub fn get_f64_by_name(&self, col_name: &str) -> Result<Option<f64>, BQError> {
        let col_index = self.fields.get(col_name);
        match col_index {
            None => Err(BQError::InvalidColumnName {
                col_name: col_name.into(),
            }),
            Some(col_index) => self.get_f64(*col_index),
        }
    }

    pub fn get_bool(&self, col_index: usize) -> Result<Option<bool>, BQError> {
        let json_value = self.get_json_value(col_index)?;
        match &json_value {
            None => Ok(None),
            Some(json_value) => match json_value {
                serde_json::Value::Bool(value) => Ok(Some(*value)),
                serde_json::Value::String(value) => {
                    let value: Result<bool, _> = value.parse();
                    match &value {
                        Err(_) => Err(BQError::InvalidColumnType {
                            col_index,
                            col_type: ResultSet::json_type(json_value),
                            type_requested: "Bool".into(),
                        }),
                        Ok(value) => Ok(Some(*value)),
                    }
                }
                _ => Err(BQError::InvalidColumnType {
                    col_index,
                    col_type: ResultSet::json_type(json_value),
                    type_requested: "Bool".into(),
                }),
            },
        }
    }

    pub fn get_bool_by_name(&self, col_name: &str) -> Result<Option<bool>, BQError> {
        let col_index = self.fields.get(col_name);
        match col_index {
            None => Err(BQError::InvalidColumnName {
                col_name: col_name.into(),
            }),
            Some(col_index) => self.get_bool(*col_index),
        }
    }

    pub fn get_string(&self, col_index: usize) -> Result<Option<String>, BQError> {
        let json_value = self.get_json_value(col_index)?;
        match json_value {
            None => Ok(None),
            Some(json_value) => match json_value {
                serde_json::Value::String(value) => Ok(Some(value)),
                serde_json::Value::Number(value) => Ok(Some(value.to_string())),
                serde_json::Value::Bool(value) => Ok(Some(value.to_string())),
                _ => Err(BQError::InvalidColumnType {
                    col_index,
                    col_type: ResultSet::json_type(&json_value),
                    type_requested: "String".into(),
                }),
            },
        }
    }

    pub fn get_string_by_name(&self, col_name: &str) -> Result<Option<String>, BQError> {
        let col_index = self.fields.get(col_name);
        match col_index {
            None => Err(BQError::InvalidColumnName {
                col_name: col_name.into(),
            }),
            Some(col_index) => self.get_string(*col_index),
        }
    }

    pub fn get_json_value(&self, col_index: usize) -> Result<Option<serde_json::Value>, BQError> {
        if self.cursor < 0 || self.cursor == self.row_count {
            return Err(BQError::NoDataAvailable);
        }
        if col_index >= self.fields.len() {
            return Err(BQError::InvalidColumnIndex { col_index });
        }

        Ok(self
            .query_response
            .rows
            .as_ref()
            .and_then(|rows| rows.get(self.cursor as usize))
            .and_then(|row| row.columns.as_ref())
            .and_then(|cols| cols.get(col_index))
            .and_then(|col| col.value.clone()))
    }

    pub fn get_json_value_by_name(
        &self,
        col_name: &str,
    ) -> Result<Option<serde_json::Value>, BQError> {
        let col_pos = self.fields.get(col_name);
        match col_pos {
            None => Err(BQError::InvalidColumnName {
                col_name: col_name.into(),
            }),
            Some(col_pos) => self.get_json_value(*col_pos),
        }
    }

    fn json_type(json_value: &serde_json::Value) -> String {
        match json_value {
            Value::Null => "Null".into(),
            Value::Bool(_) => "Bool".into(),
            Value::Number(_) => "Number".into(),
            Value::String(_) => "String".into(),
            Value::Array(_) => "Array".into(),
            Value::Object(_) => "Object".into(),
        }
    }

    // require_ methods raise an error if data is None

    pub fn require_offsetdatetime_by_name(
        &self,
        column_name: &str,
    ) -> Result<OffsetDateTime, BQError> {
        let data = self.get_i64_by_name(column_name);
        match data {
            Ok(maybe_data) => match maybe_data {
                Some(data) => Ok(OffsetDateTime::from_unix_timestamp(data)),
                None => Err(BQError::NoDataAvailable),
            },
            Err(e) => Err(e),
        }
    }

    pub fn require_commaseperatedstring_by_name(
        &self,
        column_name: &str,
    ) -> Result<String, BQError> {
        let promotion_codes_raw = self.get_json_value_by_name(column_name);
        match promotion_codes_raw {
            Ok(maybe_data) => match maybe_data {
                Some(data) => {
                    let data = data.to_string();
                    let promotion_codes: Vec<String> = serde_json::from_str(&data).unwrap();
                    Ok(promotion_codes.join(","))
                }
                None => Err(BQError::NoDataAvailable),
            },
            Err(e) => Err(e),
        }
    }

    pub fn require_string_by_name(&self, column_name: &str) -> Result<String, BQError> {
        let data = self.get_string_by_name(column_name);
        match data {
            Ok(maybe_data) => match maybe_data {
                Some(data) => Ok(data),
                None => Err(BQError::NoDataAvailable),
            },
            Err(e) => Err(e),
        }
    }

    pub fn require_i32_by_name(&self, column_name: &str) -> Result<i32, BQError> {
        let data = self.get_i64_by_name(column_name);
        match data {
            Ok(maybe_data) => match maybe_data {
                Some(data) => match i32::try_from(data) {
                    Ok(data) => Ok(data),
                    Err(_) => Err(BQError::IntegerCastUnsuccessful),
                },
                None => Err(BQError::NoDataAvailable),
            },
            Err(e) => Err(e),
        }
    }
}
