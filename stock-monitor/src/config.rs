use std::{fs, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub symbols: Vec<String>,
    pub rotation_interval_secs: u64,
    pub quote_refresh_secs: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            symbols: vec![
                "sh000001".to_owned(),
                "sz399001".to_owned(),
                "sh600519".to_owned(),
            ],
            rotation_interval_secs: 6,
            quote_refresh_secs: 12,
        }
    }
}

pub fn config_path() -> PathBuf {
    PathBuf::from("stock-monitor.json")
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    let Ok(content) = fs::read_to_string(&path) else {
        return AppConfig::default();
    };

    serde_json::from_str(&content).unwrap_or_else(|_| AppConfig::default())
}

pub fn save_config(config: &AppConfig) -> anyhow::Result<()> {
    let content = serde_json::to_string_pretty(config).context("serialize config")?;
    fs::write(config_path(), content).context("write config")?;
    Ok(())
}
