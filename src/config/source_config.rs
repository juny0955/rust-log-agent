use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SourceConfig {
    pub name: String,
    pub log_path: String,

    #[serde(default = "default_delay_ms")]
    pub delay: u64,
}

fn default_delay_ms() -> u64 { 500 }
