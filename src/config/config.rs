use crate::config::global_config::GlobalConfig;
use crate::config::source_config::SourceConfig;
use serde::Deserialize;
use std::sync::OnceLock;
use std::fs;

const CONFIG_PATH: &str = "log-agent.config";

static GLOBAL_CONFIG: OnceLock<GlobalConfig> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct Config {
    pub global: GlobalConfig,
    pub sources: Vec<SourceConfig>,
}

pub fn load_config() -> Result<Vec<SourceConfig>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(CONFIG_PATH)?;
    let config: Config = toml::from_str(&content)?;

    if GLOBAL_CONFIG.set(config.global).is_err() {
        eprintln!("GLOBAL_CONFIG is already initialized");
    } else {
        println!("{}", global_config());
        println!("=====Source Config=====");
        config.sources.iter()
            .for_each(|s| println!("{}", s));
        println!("=======================");
    }

    Ok(config.sources)
}

pub fn global_config() -> &'static GlobalConfig {
    GLOBAL_CONFIG
        .get()
        .expect("GLOBAL_CONFIG is not initialized")
}