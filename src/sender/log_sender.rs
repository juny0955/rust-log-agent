use crate::config::config::global_config;
use crate::config::global_config::SendType;
use crate::sender::http::http_sender::HttpSenderStrategy;
use crate::sender::log_data::LogData;
use std::sync::Arc;

pub trait LogSender: Send + Sync{
    fn send(&self, log_data: LogData);
}

pub fn build_sender() -> Arc<dyn LogSender> {
    let global_config = global_config();

    match global_config.send_type {
        SendType::HTTP => Arc::new(HttpSenderStrategy::new()),
    }
}