use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct LogData {
    pub name: String,
    pub data: String,
    pub timestamp: DateTime<Utc>
}

impl LogData {
    pub fn new(name: &str, data: &str) -> Self {
        Self {
            name: name.to_string(),
            data: data.to_string(),
            timestamp: Utc::now(),
        }
    }
}