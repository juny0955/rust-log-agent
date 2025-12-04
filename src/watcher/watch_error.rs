use error::Error;
use std::fmt::{Display, Formatter};
use std::io::ErrorKind;
use std::{error, io};

#[derive(Debug)]
pub enum WatchError {
    Recoverable(io::Error),
    UnRecoverable(io::Error),
}
impl Error for WatchError {}

impl WatchError {
   pub fn from_io_error(error: io::Error) -> Self {
        match error.kind() {
            ErrorKind::Interrupted | ErrorKind::WouldBlock => WatchError::Recoverable(error),
            _ => WatchError::UnRecoverable(error),
        }
    }
}

impl Display for WatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WatchError::Recoverable(e) => write!(f, "{e}"),
            WatchError::UnRecoverable(e) => write!(f, "{e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn interrupted_is_recoverable() {
        let io_error = io::Error::new(ErrorKind::Interrupted, "test interrupted");
        let watch_error = WatchError::from_io_error(io_error);

        assert!(matches!(watch_error, WatchError::Recoverable(_)));
    }

    #[test]
    pub fn would_block_is_recoverable() {
        let io_error = io::Error::new(ErrorKind::WouldBlock, "test would block");
        let watch_error = WatchError::from_io_error(io_error);

        assert!(matches!(watch_error, WatchError::Recoverable(_)));
    }

    #[test]
    pub fn not_fount_is_unrecoverable() {
        let io_error = io::Error::new(ErrorKind::NotFound, "test not found");
        let watch_error = WatchError::from_io_error(io_error);

        assert!(matches!(watch_error, WatchError::UnRecoverable(_)));
    }
}