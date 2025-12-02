use std::fmt::{Display, Formatter};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SourceConfig {
    pub name: String,
    pub log_path: String,
}

impl Display for SourceConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: {} log_path: {}", self.name, self.log_path)
    }
}