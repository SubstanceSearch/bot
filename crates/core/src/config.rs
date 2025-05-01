use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub discord: DiscordConfig,
    pub anthropic: AnthropicConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DiscordConfig {
    pub token: String,
    pub guild_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnthropicConfig {
    pub api_key: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
        
        let s = Config::builder()
            // Start with default settings
            .set_default("discord.guild_id", None::<String>)?
            // Add config file if it exists
            .add_source(File::from(Path::new(&config_path)).required(false))
            // Add environment variables with prefix "BOT_"
            .add_source(config::Environment::with_prefix("BOT").separator("_"))
            .build()?;

        s.try_deserialize()
    }
} 