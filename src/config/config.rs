use crate::config::config_error::ConfigError;
use crate::config::global_config::{GlobalConfig, SendType};
use crate::config::source_config::SourceConfig;
use serde::Deserialize;
use std::{fs, process};
use std::collections::HashSet;
use std::sync::OnceLock;
use reqwest::Url;
use tracing::{error, info, warn};

const CONFIG_PATH: &str = "log-agent.config";

static GLOBAL_CONFIG: OnceLock<GlobalConfig> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct Config {
    pub global: GlobalConfig,
    pub sources: Vec<SourceConfig>,
}

pub fn load_config() -> Result<Vec<SourceConfig>, ConfigError> {
    let config = parse_config()?;

    if GLOBAL_CONFIG.set(config.global).is_err() {
        warn!("GLOBAL_CONFIG is already initialized");
    }

    print_config(&config.sources);

    Ok(config.sources)
}

pub fn global_config() -> &'static GlobalConfig {
    GLOBAL_CONFIG.get().expect("global config is not initialized")
}

fn parse_config_from_toml(content: &str) -> Result<Config, ConfigError> {
    let config: Config = toml::from_str(&content)
        .map_err(ConfigError::CanNotParseToml)?;

    valid_config(&config)?;

    Ok(config)
}

fn parse_config() -> Result<Config, ConfigError> {
    let content = fs::read_to_string(CONFIG_PATH)
        .map_err(ConfigError::CanNotRead)?;

    parse_config_from_toml(&content)
}

fn valid_config(config: &Config) -> Result<(), ConfigError> {
    if matches!(config.global.send_type, SendType::HTTP) {
        let url = Url::parse(&config.global.end_point)
            .map_err(|_| ConfigError::InvalidEndPoint(config.global.end_point.clone()))?;

        if url.scheme() != "http" && url.scheme() != "https" {
            return Err(ConfigError::InvalidEndPoint(config.global.end_point.clone()))
        }
    }

    if config.global.retry < 1 {
        return Err(ConfigError::RetryIsUnderOne);
    }

    if config.global.channel_bound < 1 {
        return Err(ConfigError::ChannelBoundIsUnderOne)
    }

    let mut set = HashSet::new();
    for s in &config.sources {
        if !set.insert(&s.name) {
            return Err(ConfigError::DuplicateSourceName(s.name.to_string()));
        }
    }

    Ok(())
}

fn print_config(sources: &[SourceConfig]) {
    let global = global_config();
    info!("Configuration Loaded Successfully");
    info!("----------------------------------");
    info!("Global:");
    info!("\t* EndPoint: {}", global.end_point);
    info!("\t* SendType: {:?}", global.send_type);
    info!("\t* Retry: {}", global.retry);
    info!("\t* Retry Delay: {}ms", global.retry_delay_ms);
    info!("\t* Channel Bound: {}", global.channel_bound);
    info!("Sources ({}):", sources.len());
    sources.iter().enumerate().for_each(|(i, s)| {
        info!("\t{}. {}", i+1, s.name);
        info!("\t\t* Path: {}", s.log_path);
        info!("\t\t* Delay: {}ms", s.delay_ms);
    });
    info!("----------------------------------");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_from_toml_test() {
        let example = r#"
            [global]
            end_point = "http://localhost:8080/log"
            send_type = "HTTP"

            [[sources]]
            name = "app1"
            log_path = "app1.log"

            [[sources]]
            name = "app2"
            log_path = "app2.log"
        "#;

        let config = parse_config_from_toml(example)
            .expect("parse err");

        assert_eq!(config.global.end_point, "http://localhost:8080/log");
        assert_eq!(config.sources.len(), 2);
    }

    #[test]
    fn parse_config_toml_invalid() {
        let example = r#"
            [global
            end_point = "http://localhost:8080/log"
            send_type = "HTTP"
        "#;

        let result = parse_config_from_toml(example);
        assert!(matches!(result, Err(ConfigError::CanNotParseToml(_))));
    }

    #[test]
    fn parse_config_toml_miss_required_field() {
        let example = r#"
            [global]
            send_type = "HTTP"
        "#;

        let result = parse_config_from_toml(example);
        assert!(matches!(result, Err(ConfigError::CanNotParseToml(_))));
    }

    #[test]
    fn retry_count_is_must_be_over_1() {
        let example = r#"
            [global]
            end_point = "http://localhost:8080/log"
            send_type = "HTTP"
            retry = 0

            [[sources]]
            name = "app1"
            log_path = "app1.log"

            [[sources]]
            name = "app1"
            log_path = "app2.log"
        "#;

        let result = parse_config_from_toml(example);
        assert!(matches!(result, Err(ConfigError::RetryIsUnderOne)));
    }

    #[test]
    fn source_name_is_must_be_unique() {
        let example = r#"
            [global]
            end_point = "http://localhost:8080/log"
            send_type = "HTTP"

            [[sources]]
            name = "app1"
            log_path = "app1.log"

            [[sources]]
            name = "app1"
            log_path = "app2.log"
        "#;

        let result = parse_config_from_toml(example);
        assert!(matches!(result, Err(ConfigError::DuplicateSourceName(_))));
    }
}