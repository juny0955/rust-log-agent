use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum SendType {
    HTTP,
}

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    pub agent_name: String,
    pub end_point: String,
    pub send_type: SendType,

    #[serde(default = "default_max_send_task")]
    pub max_send_task: u8,

    #[serde(default = "default_retry_count")]
    pub retry_count: u32,

    #[serde(default = "default_retry_delay_ms")]
    pub retry_delay_ms: u64,

    #[serde(default = "default_channel_bound")]
    pub channel_bound: usize,

    #[serde(default = "default_interval_secs")]
    pub interval_secs: u64,

    #[serde(default = "default_max_batch_size")]
    pub max_batch_size: u8,
}

fn default_max_send_task() -> u8 { 5 }
fn default_retry_count() -> u32 {
    3
}
fn default_retry_delay_ms() -> u64 {
    100
}
fn default_channel_bound() -> usize {
    1024
}
fn default_interval_secs() -> u64 {
    5
}
fn default_max_batch_size() -> u8 {
    100
}
