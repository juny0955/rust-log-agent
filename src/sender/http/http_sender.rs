use crate::config::config::global_config;
use crate::sender::http::log_body::LogBody;
use crate::sender::log_sender::LogSender;
use reqwest::blocking::Client;
use reqwest::Error;
use std::thread;
use std::time::Duration;
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

    fn is_retryable(error: &Error) -> bool {
        error.is_timeout() || error.is_connect()
    }
}

impl LogSender for HttpSenderStrategy {
    fn send(&self, name: &str, data: &str) {
        let body = LogBody::new(name, data);
        let global_config = global_config();
        let max_retry = global_config.retry;
        let endpoint = &global_config.end_point;

        for attempt in 1..=max_retry {
            let result = self.client
                .post(endpoint)
                .json(&body)
                .send();


            match result {
                Ok(response) => {
                    if response.status().is_success() {
                        debug!("{name} send success. status: {}, attempt: {attempt}/{max_retry}", response.status());
                        return;
                    }

                    error!("{name} send failed. status: {}", response.status());
                },
                Err(e) => {
                    if !Self::is_retryable(&e) {
                        error!("{name} send failed(non-retry): {e}");
                        return;
                    }

                    warn!("{name} send failed: {e}, retry...{attempt}/{max_retry}");
                }
            }

            if attempt < max_retry {
                thread::sleep(Duration::from_millis(global_config.retry_delay_ms));
            }
        }
    }
}
