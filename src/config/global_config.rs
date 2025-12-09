use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum SendType {
    HTTP,
}

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    pub end_point: String,
    pub send_type: SendType,

    #[serde(default = "default_retry")]
    pub retry: u32,

    #[serde(default = "default_retry_delay_ms")]
    pub retry_delay_ms: u64,

    #[serde(default = "default_channel_bound")]
    pub channel_bound: usize,
}

fn default_retry() -> u32 { 3 }
fn default_retry_delay_ms() -> u64 { 100 }
fn default_channel_bound() -> usize { 1024 }