use crate::config::config::global_config;
use crate::sender::http::http_error::HttpError;
use crate::sender::log_data::LogData;
use crate::sender::log_sender::LogSender;
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;
use tokio::time;
use tracing::{debug, error, warn};

pub struct HttpSenderStrategy {
    client: Client,
}

impl HttpSenderStrategy {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP Client.");

        Self { client }
    }

    async fn try_send(&self, end_point: &str, log_data: &LogData) -> Result<(), HttpError> {
        let response = self.client
            .post(end_point)
            .json(log_data)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(response.status().into())
        }
    }
}

#[async_trait]
impl LogSender for HttpSenderStrategy {
    async fn send(&self, log_data: LogData) {
        let global_config = global_config();
        let endpoint = &global_config.end_point;
        let max_retry = global_config.retry;
        let retry_delay = Duration::from_millis(global_config.retry_delay_ms);

        for attempt in 1..=max_retry {
            match self.try_send(endpoint, &log_data).await {
                Ok(()) => debug!("{} send success. on attempt: {attempt}/{max_retry}", log_data.name),
                Err(HttpError::NonRetryable(e)) => error!("{} send failed(non-retry): {e}", log_data.name),
                Err(HttpError::Retryable(e)) => {
                    if attempt == max_retry {
                        error!("{} send failed after {max_retry} msg: {e}", log_data.name);
                        return;
                    }

                    warn!("{} send failed: {e}, retry...{attempt}/{max_retry}", log_data.name);
                    time::sleep(retry_delay).await;
                },
            }
        }
    }
}
