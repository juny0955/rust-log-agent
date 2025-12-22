use crate::config::GlobalConfig;
use crate::sender::payload::Payload;
use crate::sender::SenderError;
use crate::sender::Sender;
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;
use tracing::{error, trace, warn};

mod http_error;
use self::http_error::HttpError;

pub struct HttpSenderStrategy {
    client: Client,
    endpoint: String,
    max_retry: u32,
    retry_delay: Duration,
}

impl HttpSenderStrategy {
    pub fn build(global_config: &GlobalConfig) -> Result<Self, SenderError> {
        let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

        Ok(Self {
            client,
            endpoint: global_config.end_point.clone(),
            max_retry: global_config.retry_count,
            retry_delay: Duration::from_millis(global_config.retry_delay_ms),
        })
    }

    // reqwest is 4xx, 5xx error not return reqwest::Error
    // use error_for_status() then mapping reqwest::Error
    async fn try_send(&self, payload: &Payload) -> Result<(), HttpError> {
        self.client
            .post(&self.endpoint)
            .json(payload)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[async_trait]
impl Sender for HttpSenderStrategy {
    async fn send(&self, payload: &Payload) {
        for attempt in 1..=self.max_retry {
            match self.try_send(payload).await {
                Ok(()) => {
                    trace!("HTTP send success. attempt {attempt}/{}", self.max_retry);
                    return;
                }
                Err(HttpError::NonRetryable(e)) => {
                    error!("HTTP send failed (non-retryable): {e}");
                    return;
                }
                Err(HttpError::Retryable(e)) => {
                    if attempt == self.max_retry {
                        error!("HTTP send failed after {} retries: {e}", self.max_retry);
                        return;
                    }

                    warn!("HTTP send failed (retryable): {e}. retry... {attempt}/{}", self.max_retry);
                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
    }
}
