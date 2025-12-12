use self::http_sender::HttpSenderStrategy;
use self::ws_sender::WebSocketSenderStrategy;
use crate::config::{global_config, SendType};
use async_trait::async_trait;

pub mod log_data;
pub use log_data::LogData;

mod http_sender;
mod ws_sender;

#[async_trait]
pub trait Sender: Send {
    async fn send(&mut self, log_data: LogData);
}

pub async fn build_sender() -> Result<Box<dyn Sender>, String> {
    let global_config = global_config();

    match global_config.send_type {
        SendType::HTTP => Ok(Box::new(HttpSenderStrategy::build())),
        SendType::WS => {
            match WebSocketSenderStrategy::build().await {
                Ok(ws_sender) => Ok(Box::new(ws_sender)),
                Err(_) => panic!(),
            }
        },
    }
}