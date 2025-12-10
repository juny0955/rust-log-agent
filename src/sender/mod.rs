use self::http_sender::HttpSenderStrategy;
use crate::config::{global_config, SendType};
use async_trait::async_trait;

pub mod log_data;
mod http_sender;
pub use log_data::LogData;

#[async_trait]
pub trait Sender: Send + Sync{
    async fn send(&self, log_data: LogData);
}

pub fn build_sender() -> Box<dyn Sender> {
    let global_config = global_config();

    match global_config.send_type {
        SendType::HTTP => Box::new(HttpSenderStrategy::new()),
    }
}