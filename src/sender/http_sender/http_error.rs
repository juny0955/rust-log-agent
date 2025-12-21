pub enum HttpError {
    Retryable(reqwest::Error),
    NonRetryable(reqwest::Error),
}

impl From<reqwest::Error> for HttpError {
    fn from(value: reqwest::Error) -> Self {
        // when timeout or connection error do retry
        if value.is_timeout() || value.is_connect() {
            return HttpError::Retryable(value);
        }

        // server error or 429(to many requests) do retry
        if let Some(status) = value.status() {
            return match status.as_u16() {
                500..=599 | 429 => HttpError::Retryable(value),
                _ => HttpError::NonRetryable(value)
            }
        }

        HttpError::NonRetryable(value)
    }
}
