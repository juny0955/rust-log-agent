use crate::{
    config::global_config,
    sender::{
        payload::Payload,
        Sender,
        SenderError
    }
};
use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, trace, warn};

mod http_error;
use self::http_error::HttpError;

pub struct HttpSenderStrategy {
    client: Client, // already use Arc
    endpoint: String,
    max_retry_count: u8,
    retry_delay: Duration,
    retry_sender: mpsc::Sender<RetryPayload>,
}

struct RetryPayload {
    payload: Arc<Payload>,
    attempt: u8,
}

impl RetryPayload {
    pub fn new(payload: Arc<Payload>) -> Self {
        Self {
            payload,
            attempt: 1,
        }
    }
}

impl HttpSenderStrategy {
    pub fn build() -> Result<Self, SenderError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        let global_config = global_config();

        let (retry_sender, retry_receiver) = mpsc::channel::<RetryPayload>(global_config.channel_bound);

        let strategy = Self {
            client: client.clone(),
            endpoint: global_config.end_point.clone(),
            max_retry_count: global_config.retry_count,
            retry_delay: Duration::from_millis(global_config.retry_delay_ms),
            retry_sender,
        };

        strategy.spawn_retry_task(retry_receiver, global_config.max_send_task);

        Ok(strategy)
    }

    fn spawn_retry_task(&self, retry_receiver: mpsc::Receiver<RetryPayload>, max_task_count: u8) {
        let retry_receiver = Arc::new(Mutex::new(retry_receiver));

        for _ in 0..max_task_count {
            let client = self.client.clone();
            let endpoint = self.endpoint.clone();
            let max_retry_count = self.max_retry_count;
            let retry_delay = self.retry_delay;

            let retry_receiver = retry_receiver.clone();

            tokio::spawn(
                Self::retry_worker_loop(
                    retry_receiver,
                    client,
                    endpoint,
                    max_retry_count,
                    retry_delay
                )
            );
        }
    }

    async fn retry_worker_loop(
        retry_receiver: Arc<Mutex<mpsc::Receiver<RetryPayload>>>,
        client: Client,
        endpoint: String,
        max_retry_count: u8,
        retry_delay: Duration,
    ) {
        loop {
            // TODO receiver.recv() is not parallelism should be remove mutex.. but how?
            let retry_payload = {
                let mut retry_receiver = retry_receiver.lock().await;
                match retry_receiver.recv().await {
                    Some(retry_payload) => retry_payload,
                    None => {
                        error!("Failed to retry channel close");
                        break;
                    },
                }
            };

            Self::process_retry(
                &client,
                &endpoint,
                retry_payload,
                max_retry_count,
                retry_delay
            ).await;
        }
    }

    async fn process_retry(
        client: &Client,
        endpoint: &str,
        mut retry_payload: RetryPayload,
        max_retry_count: u8,
        retry_delay: Duration,
    ) {
        while retry_payload.attempt < max_retry_count {
            let backoff = Self::calc_backoff(retry_delay, retry_payload.attempt);
            tokio::time::sleep(backoff).await;
            retry_payload.attempt += 1;

            match Self::try_send(&client, &endpoint, retry_payload.payload.as_ref()).await {
                Ok(()) => {
                    debug!("HTTP retry success. attempt {}/{max_retry_count}", retry_payload.attempt);
                    return;
                }
                Err(HttpError::NonRetryable(e)) => {
                    error!("HTTP retry failed (non-retryable) attempt {}/{max_retry_count}: {e}", retry_payload.attempt);
                    return;
                }
                Err(HttpError::Retryable(e)) => warn!("HTTP retry failed (retryable) attempt {}/{max_retry_count}: {e}", retry_payload.attempt),
            }
        }

        error!("HTTP retry failed after {} attempts (max: {max_retry_count})", retry_payload.attempt);
    }

    fn calc_backoff(base_delay: Duration, attempt: u8) -> Duration {
        const MAX_DELAY: Duration = Duration::from_secs(30);

        // base_delay * 2^(attempt-1)
        let backoff = base_delay * 2_u32.pow((attempt - 1) as u32);
        if MAX_DELAY > backoff { backoff } else { MAX_DELAY }
    }

    // reqwest is 4xx, 5xx error not return reqwest::Error
    // use error_for_status() then mapping reqwest::Error
    async fn try_send(client: &Client, endpoint: &str, payload: &Payload) -> Result<(), HttpError> {
        client.post(endpoint)
            .json(payload)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[async_trait]
impl Sender for HttpSenderStrategy {
    async fn send(&self, payload: Payload) {
        let payload = Arc::new(payload);
        let endpoint = self.endpoint.as_str();

        match Self::try_send(&self.client, endpoint, payload.as_ref()).await {
            Ok(()) => trace!("HTTP send success."),
            Err(HttpError::NonRetryable(e)) => error!("HTTP send failed (non-retryable): {e}"),
            Err(HttpError::Retryable(e)) => {
                warn!("HTTP send failed (retryable) attempt 1/{}: {e}", self.max_retry_count);

                if let Err(e) = self.retry_sender.send(RetryPayload::new(payload)).await {
                    error!("Failed to retry channel close: {e}");
                }
            }
        }
    }
}
