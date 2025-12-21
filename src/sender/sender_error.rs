use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum SenderError {
    HttpClientBuildError(reqwest::Error),
    SerializedError(serde_json::Error),
}

impl From<reqwest::Error> for SenderError {
    fn from(value: reqwest::Error) -> Self {
        SenderError::HttpClientBuildError(value)
    }
}

impl From<serde_json::Error> for SenderError {
    fn from(value: serde_json::Error) -> Self {
        SenderError::SerializedError(value)
    }
}

impl Display for SenderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SenderError::HttpClientBuildError(e) => write!(f, "Failed to build HTTP client: {e}"),
            SenderError::SerializedError(e) => write!(f, "Cannot serialized data: {e}"),
        }
    }
}
