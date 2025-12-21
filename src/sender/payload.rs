use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{config::global_config, log_event::LogEvent};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub agent_name: String,
    pub sources: Vec<Source>,
}

impl Payload {
    pub fn new(sources: Vec<Source>) -> Self {
        Self {
            agent_name: global_config().agent_name.clone(),
            sources,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub source_name: String,
    pub logs: Vec<Logs>,
}

impl Source {
    pub fn new(source_name: String, logs: Vec<Logs>) -> Self {
        Self { source_name, logs }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Logs {
    pub data: String,
    pub timestamp: DateTime<Utc>,
}

impl Logs {
    pub fn from_event(log_event: LogEvent) -> Self {
        Self {
            data: log_event.log,
            timestamp: log_event.timestamp,
        }
    }
}
