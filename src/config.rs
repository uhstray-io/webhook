use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub discord: Vec<WebhookConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub port: String,
    pub host: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WebhookConfig {
    pub name: String,
    pub path: String,
    pub url: String,
    pub logging: Option<bool>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
        let config_file = std::fs::File::open(path)?;
        let cfg = serde_yaml::from_reader(config_file)?;
        Ok(cfg)
    }

    // Return a Shared Config
    pub fn shared(self) -> std::sync::Arc<Self> {
        std::sync::Arc::new(self)
    }
}
