use tokio_tungstenite::tungstenite::Error;
use crate::sender::ws_sender::ws_error::WsError::{NonRetryable, Retryable};

pub enum WsError {
    Retryable(String),
    NonRetryable(String),
}

impl From<Error> for WsError {
    fn from(value: Error) -> Self {
        match value {
            Error::Io(e) => Retryable(e.to_string()),
            Error::Http(response) => {
                let status = response.status();
                if status.is_server_error() {
                    return Retryable(format!("status: {status}"))
                }

                NonRetryable(format!(""))
            }
            other => NonRetryable(format!("{other}"))
        }
    }
}