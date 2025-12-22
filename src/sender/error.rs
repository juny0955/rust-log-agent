use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum SenderError {
    SenderFailedBuild(reqwest::Error),
    SerializedError(serde_json::Error),
}

impl From<reqwest::Error> for SenderError {
    fn from(value: reqwest::Error) -> Self {
        SenderError::SenderFailedBuild(value)
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
            SenderError::SenderFailedBuild(e) => write!(f, "Failed to build sender: {e}"),
            SenderError::SerializedError(e) => write!(f, "Cannot serialized data: {e}"),
        }
    }
}
