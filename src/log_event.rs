use chrono::{DateTime, Utc};

pub struct LogEvent {
    pub name: String,
    pub log: String,
    pub timestamp: DateTime<Utc>,
}

impl LogEvent {
    pub fn new(name: String, log: String) -> Self {
        Self {
            name,
            log,
            timestamp: Utc::now(),
        }
    }
}
