use std::fmt::{Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum ConfigError {
    CanNotRead(io::Error),
    CanNotParseToml(toml::de::Error),
    UrlParseError(url::ParseError),
    InvalidEndPoint(String),
    SendTaskIsUnderOne,
    RetryCountIsUnderOne,
    ChannelBoundIsUnderOne,
    DuplicateSourceName(String),
    DuplicateLogPath(String),
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        ConfigError::CanNotParseToml(value)
    }
}

impl From<url::ParseError> for ConfigError {
    fn from(value: url::ParseError) -> Self {
        ConfigError::UrlParseError(value)
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::CanNotRead(e) => write!(f, "Failed to read config file: {}", e),
            ConfigError::CanNotParseToml(e) => write!(f, "Failed to Parse TOML: {}", e),
            ConfigError::UrlParseError(e) => write!(f, "Cannot Parse endpoint: {e}"),
            ConfigError::InvalidEndPoint(end_point) => write!(f, "Invalid endpoint {end_point}"),
            ConfigError::SendTaskIsUnderOne => write!(f, "send task is must be over 1"),
            ConfigError::RetryCountIsUnderOne => write!(f, "Retry count is must be over 1"),
            ConfigError::ChannelBoundIsUnderOne => write!(f, "Channel bound is must be over 1"),
            ConfigError::DuplicateSourceName(name) => write!(f, "Duplicated source name in config: '{name}'"),
            ConfigError::DuplicateLogPath(path) => write!(f, "Duplicated log file path in config: '{path}'")
        }
    }
}
