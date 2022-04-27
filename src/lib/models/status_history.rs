use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{from_value, json, Value as JsonValue};
use strum_macros::{Display as EnumToString, EnumString};
use time::OffsetDateTime;

use crate::{error, telemetry::LogKey};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, EnumToString, EnumString)]
pub enum Status {
    NotReported,
    Reported,
    WillNotReport,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusHistoryEntry {
    pub t: OffsetDateTime,
    pub status: Status,
}
impl PartialEq for StatusHistoryEntry {
    fn eq(&self, other: &Self) -> bool {
        self.status == other.status && self.t.unix_timestamp() == other.t.unix_timestamp()
    }
}
impl Eq for StatusHistoryEntry {}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusHistory {
    pub entries: Vec<StatusHistoryEntry>,
}

impl StatusHistory {
    pub fn from_json_value(v: JsonValue) -> StatusHistory {
        let status_history: StatusHistory = match from_value(v.clone()) {
            Ok(v) => v,
            Err(e) => {
                error!(
                    LogKey::StatusHistoryDeserializeError,
                    error = e,
                    v = v.to_string().as_str(),
                    "Error deserializing status_history"
                );
                StatusHistory { entries: vec![] }
            }
        };
        status_history
    }
}

pub trait UpdateStatus {
    fn get_status_t(&self) -> Option<OffsetDateTime>;
    fn get_raw_status(&self) -> Option<String>;
    fn get_raw_status_history(&self) -> Option<JsonValue>;
    fn set_status_t(&mut self, v: Option<OffsetDateTime>);
    fn set_raw_status(&mut self, v: Option<String>);
    fn set_raw_status_history(&mut self, v: Option<JsonValue>);

    fn get_status(&self) -> Option<Status> {
        let status_value = self.get_raw_status().unwrap_or_else(|| "".to_string());
        match Status::from_str(&status_value) {
            Ok(status) => Some(status),
            Err(_) => None,
        }
    }

    fn get_status_history(&self) -> Option<StatusHistory> {
        self.get_raw_status_history()
            .map(StatusHistory::from_json_value)
    }

    fn update_status(&mut self, new_status: Status) {
        let t = OffsetDateTime::now_utc();
        self.set_status_t(Some(t));
        self.set_raw_status(Some(new_status.to_string()));
        let mut status_history = match self.get_status_history() {
            Some(v) => v,
            None => StatusHistory { entries: vec![] },
        };
        status_history.entries.push(StatusHistoryEntry {
            status: new_status,
            t,
        });
        self.set_raw_status_history(Some(json!(status_history)));
    }
}
