use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Semaphore;
use tokio::task;

use crate::config::global_config;

pub mod payload;
use self::payload::Payload;

mod error;
pub use error::SenderError;

mod strategies;

#[async_trait]
pub trait Sender: Send + Sync {
    async fn send(&self, payload: Payload);
}
pub fn spawn_sender(mut payload_receiver: Receiver<Payload>) -> Result<task::JoinHandle<()>, SenderError> {
    let sender = strategies::build_sender()?;
    let semaphore = Arc::new(Semaphore::new(global_config().max_send_task as usize));

    let handle = tokio::spawn(async move {
        while let Some(payload) = payload_receiver.recv().await {
            let permit = match semaphore.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => break,
            };

            let sender = sender.clone();
            tokio::spawn(async move {
                let _permit = permit;
                let _ = sender.send(payload).await;
            });
        }
    });

    Ok(handle)
}
