use error::Error;
use std::fmt::{Display, Formatter};
use std::{error, io};

#[derive(Debug)]
pub enum ConfigError {
    CanNotRead(io::Error),
    CanNotParseToml(toml::de::Error),
    RetryIsUnderOne,
}
impl Error for ConfigError {}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::CanNotRead(e) => write!(f, "Failed to read config file: {}", e.to_string()),
            ConfigError::CanNotParseToml(e) => write!(f, "Failed to Parse TOML: {}", e.to_string()),
            ConfigError::RetryIsUnderOne => write!(f, "Retry count is must over 1"),
        }
    }
}