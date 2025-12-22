use crate::config::global_config;
use crate::sender::payload::Payload;
use crate::sender::Sender;
use crate::sender::SenderError;
use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info_span, trace, warn};

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

        for task_id in 0..=max_task_count {
            let client = self.client.clone();
            let endpoint = self.endpoint.clone();
            let max_retry_count = self.max_retry_count;
            let retry_delay = self.retry_delay;

            let retry_receiver = retry_receiver.clone();

            tokio::spawn(async move {
                let span = info_span!("retry-send-task-{}", task_id);
                let _ = span.enter();

                loop {
                    let retry_payload = {
                        let mut retry_receiver = retry_receiver.lock().await;
                        match retry_receiver.recv().await {
                            Some(retry_payload) => retry_payload,
                            None => break,
                        }
                    };

                    let payload = retry_payload.payload;
                    let mut attempt = retry_payload.attempt;

                    while attempt < max_retry_count {
                        tokio::time::sleep(retry_delay).await;
                        attempt += 1;

                        match Self::try_send(&client, &endpoint, payload.as_ref()).await {
                            Ok(()) => {
                                trace!("HTTP retry success. attempt {}/{}", attempt, max_retry_count);
                                return;
                            }
                            Err(HttpError::NonRetryable(e)) => {
                                error!("HTTP retry failed (non-retryable) attempt {}/{}: {e}", attempt, max_retry_count);
                                return;
                            }
                            Err(HttpError::Retryable(e)) => {
                                if attempt >= max_retry_count {
                                    error!("HTTP retry failed after {} retries: {e}", max_retry_count);
                                    return;
                                } else {
                                    warn!("HTTP retry failed (retryable) attempt {}/{}: {e}", attempt, max_retry_count);
                                    attempt += 1;
                                }
                            }
                        }
                    }
                }
            });
        }
    }

    // reqwest is 4xx, 5xx error not return reqwest::Error
    // use error_for_status() then mapping reqwest::Error
    async fn try_send(client: &Client, endpoint: &str, payload: &Payload) -> Result<(), HttpError> {
        client
            .post(endpoint)
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
                warn!("HTTP send failed (retryable): {e}, 1/{}", self.max_retry_count);

                let retry_payload = RetryPayload {
                    payload,
                    attempt: 1,
                };

                let _ = self.retry_sender.send(retry_payload).await;
            }
        }
    }
}
