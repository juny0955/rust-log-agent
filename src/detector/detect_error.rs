use error::Error;
use std::fmt::{Display, Formatter};
use std::io::ErrorKind;
use std::{error, io};

#[derive(Debug)]
pub enum DetectError {
    Recoverable(io::Error),
    UnRecoverable(io::Error),
    ChannelClosed,
}
impl Error for DetectError {}

impl DetectError {
   pub fn from_io_error(error: io::Error) -> Self {
        match error.kind() {
            ErrorKind::Interrupted | ErrorKind::WouldBlock => DetectError::Recoverable(error),
            _ => DetectError::UnRecoverable(error),
        }
    }
}

impl Display for DetectError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DetectError::Recoverable(e) => write!(f, "{e}"),
            DetectError::UnRecoverable(e) => write!(f, "{e}"),
            DetectError::ChannelClosed => write!(f, "receiver channel closed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupted_is_recoverable() {
        let io_error = io::Error::new(ErrorKind::Interrupted, "test interrupted");
        let watch_error = DetectError::from_io_error(io_error);

        assert!(matches!(watch_error, DetectError::Recoverable(_)));
    }

    #[test]
    fn would_block_is_recoverable() {
        let io_error = io::Error::new(ErrorKind::WouldBlock, "test would block");
        let watch_error = DetectError::from_io_error(io_error);

        assert!(matches!(watch_error, DetectError::Recoverable(_)));
    }

    #[test]
    fn not_fount_is_unrecoverable() {
        let io_error = io::Error::new(ErrorKind::NotFound, "test not found");
        let watch_error = DetectError::from_io_error(io_error);

        assert!(matches!(watch_error, DetectError::UnRecoverable(_)));
    }
}