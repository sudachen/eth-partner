//! Configuration management for the REPL application.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// Represents the overall application configuration.
#[derive(Deserialize, Debug, Default)]
pub struct Config {
    /// LLM provider settings.
    #[serde(default)]
    pub llm: LlmConfig,
}

/// Configuration specific to the LLM provider.
#[derive(Deserialize, Debug, Default)]
pub struct LlmConfig {
    /// The Google API key for Gemini.
    pub google_api_key: Option<String>,
}

/// Loads the application configuration.
///
/// The configuration is loaded from `~/.config/eth-partner/config.json`.
/// If the file does not exist, a default configuration is returned.
pub fn load() -> Result<Config> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let config_content = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file at {:?}", config_path))?;

    let config: Config = serde_json::from_str(&config_content)
        .with_context(|| "Failed to parse config file")?;

    Ok(config)
}

/// Returns the path to the configuration file.
///
/// The path is `~/.config/eth-partner/config.json`.
fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Failed to find user's config directory")?
        .join("eth-partner");

    // Ensure the directory exists.
    fs::create_dir_all(&config_dir)
        .with_context(|| format!("Failed to create config directory at {:?}", config_dir))?;

    Ok(config_dir.join("config.json"))
}
