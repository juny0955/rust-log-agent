use error::Error;
use std::fmt::{Display, Formatter};
use std::{error, io};

#[derive(Debug)]
pub enum ConfigError {
    CanNotRead(io::Error),
    CanNotParseToml(toml::de::Error),
    InvalidEndPoint(String),
    RetryIsUnderOne,
    ChannelBoundIsUnderOne,
    DuplicateSourceName(String),
}
impl Error for ConfigError {}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::CanNotRead(e) => write!(f, "Failed to read config file: {}", e.to_string()),
            ConfigError::CanNotParseToml(e) => write!(f, "Failed to Parse TOML: {}", e.to_string()),
            ConfigError::InvalidEndPoint(end_point) => write!(f, "Invalid endpoint {end_point}"),
            ConfigError::RetryIsUnderOne => write!(f, "Retry count is must over 1"),
            ConfigError::ChannelBoundIsUnderOne => write!(f, "Channel bound is must over 1"),
            ConfigError::DuplicateSourceName(name) => write!(f, "Duplicated source name in config: '{name}', source name is must be unique."),
        }
    }
}