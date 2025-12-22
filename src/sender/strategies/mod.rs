use self::http::HttpSenderStrategy;
use super::{Sender, SenderError};
use crate::config::{global_config, SendType};
use std::sync::Arc;

mod http;

pub fn build_sender() -> Result<Arc<dyn Sender>, SenderError> {
    match global_config().send_type {
        SendType::HTTP => Ok(Arc::new(HttpSenderStrategy::build()?)),
    }
}