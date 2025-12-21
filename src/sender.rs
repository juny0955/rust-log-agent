use crate::{
    config::{global_config, SendType},
    sender::payload::Payload,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Semaphore;
use tokio::task;

pub mod payload;

mod sender_error;
pub use sender_error::SenderError;

mod http_sender;
mod retry_payload;

use self::http_sender::HttpSenderStrategy;

#[async_trait]
pub trait Sender: Send + Sync {
    async fn send(&self, payload: &Payload);
}

fn build_sender() -> Result<Arc<dyn Sender>, SenderError> {
    let global_config = global_config();

    match global_config.send_type {
        SendType::HTTP => Ok(Arc::new(HttpSenderStrategy::build()?)),
    }
}

pub async fn spawn_sender(mut payload_receiver: Receiver<Payload>) -> Result<task::JoinHandle<()>, SenderError> {
    let sender = build_sender()?;
    let semaphore = Arc::new(Semaphore::new(global_config().max_send_task as usize));

    let handle = tokio::spawn(async move {
        loop {
            // 1. get a permit
            let permit = match semaphore.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => break,
            };

            // 2. receive a message
            let payload = match payload_receiver.recv().await {
                Some(payload) => payload,
                None => {
                    drop(permit);
                    break;
                }
            };

            // 3. create a task to send
            let sender = sender.clone();
            tokio::spawn(async move {
                sender.send(&payload).await;
                drop(permit);
            });
        }
    });

    Ok(handle)
}
