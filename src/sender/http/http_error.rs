use reqwest::{Error, StatusCode};

#[derive(Debug)]
pub enum HttpError {
    Retryable(String),
    NonRetryable(String),
}

impl From<Error> for HttpError {
    fn from(value: Error) -> Self {
        // when timeout or connection error do retry
        if value.is_timeout() || value.is_connect() {
            HttpError::Retryable(value.to_string())
        } else {
            HttpError::NonRetryable(value.to_string())
        }
    }
}

impl From<StatusCode> for HttpError {
    fn from(value: StatusCode) -> Self {
        // only server error do retry
        if value.is_server_error() {
            HttpError::Retryable("{value}".to_string())
        } else {
            HttpError::NonRetryable("{value}".to_string())
        }
    }
}
