use std::fmt::{Display, Formatter};
use std::io;
use std::io::ErrorKind;
use tokio::sync::mpsc;

use crate::log_event::LogEvent;

#[derive(Debug)]
pub enum DetectError {
    Recoverable(io::Error),
    UnRecoverable(io::Error),
    ChannelClosed(mpsc::error::SendError<LogEvent>),
}

impl From<io::Error> for DetectError {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            ErrorKind::Interrupted | ErrorKind::WouldBlock => DetectError::Recoverable(value),
            _ => DetectError::UnRecoverable(value),
        }
    }
}

impl From<mpsc::error::SendError<LogEvent>> for DetectError {
    fn from(value: mpsc::error::SendError<LogEvent>) -> Self {
        DetectError::ChannelClosed(value)
    }
}

impl Display for DetectError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DetectError::Recoverable(e) => write!(f, "{e}"),
            DetectError::UnRecoverable(e) => write!(f, "{e}"),
            DetectError::ChannelClosed(e) => write!(f, "receiver channel closed: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupted_is_recoverable() {
        let io_error = io::Error::new(ErrorKind::Interrupted, "test interrupted");
        let watch_error = DetectError::from(io_error);

        assert!(matches!(watch_error, DetectError::Recoverable(_)));
    }

    #[test]
    fn would_block_is_recoverable() {
        let io_error = io::Error::new(ErrorKind::WouldBlock, "test would block");
        let watch_error = DetectError::from(io_error);

        assert!(matches!(watch_error, DetectError::Recoverable(_)));
    }

    #[test]
    fn not_found_is_unrecoverable() {
        let io_error = io::Error::new(ErrorKind::NotFound, "test not found");
        let watch_error = DetectError::from(io_error);

        assert!(matches!(watch_error, DetectError::UnRecoverable(_)));
    }
}
