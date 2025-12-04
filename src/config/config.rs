use crate::config::config_error::ConfigError;
use crate::config::global_config::GlobalConfig;
use crate::config::source_config::SourceConfig;
use serde::Deserialize;
use std::{fs, process};
use std::sync::OnceLock;
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
    GLOBAL_CONFIG.get().unwrap_or_else(|| {
        error!("global config is not initialized");
        process::exit(1);
    })
}

fn parse_config_from_toml(content: &str) -> Result<Config, ConfigError> {
    let config: Config = toml::from_str(&content)
        .map_err(ConfigError::CanNotParseToml)?;

    if config.global.retry < 1 {
        return Err(ConfigError::RetryIsUnderOne);
    }

    Ok(config)
}

fn parse_config() -> Result<Config, ConfigError> {
    let content = fs::read_to_string(CONFIG_PATH)
        .map_err(ConfigError::CanNotRead)?;

    parse_config_from_toml(&content)
}

fn print_config(sources: &[SourceConfig]) {
    let global = global_config();
    info!("Configuration Loaded Successfully");
    info!("----------------------------------");
    info!("Global:");
    info!("\t* EndPoint: {}", global.end_point);
    info!("\t* SendType: {:?}", global.send_type);
    info!("\t* Retry: {}", global.retry);
    info!("\t* Retry_Delay: {}ms", global.retry_delay_ms);
    info!("Sources ({}):", sources.len());
    sources.iter().enumerate().for_each(|(i, s)| {
        info!("\t{}. {}", i+1, s.name);
        info!("\t\t* Path: {}", s.log_path);
    });
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
}