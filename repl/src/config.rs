//! Configuration management for the REPL application.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents the overall application configuration.
#[derive(Deserialize, Debug, Default, PartialEq)]
pub struct Config {
    /// LLM provider settings.
    #[serde(default)]
    pub llm: LlmConfig,

    /// Tool settings.
    #[serde(default)]
    pub tools: ToolsConfig,

    /// Embedded MCP wallet server settings.
    #[serde(default)]
    pub wallet_server: WalletServerConfig,
}

/// Apply environment variables as defaults. Config file values take precedence.
fn apply_env_defaults(cfg: &mut Config) {
    // Wallet server env defaults
    // Only override when current values are defaults (for String) or None (for Option types).
    if let Ok(v) = env::var("ETH_RPC_URL") {
        // Treat the compile-time default as a sentinel that may be replaced by env.
        if cfg.wallet_server.rpc_url == "http://127.0.0.1:8545" {
            cfg.wallet_server.rpc_url = v;
        }
    }

    if let Ok(v) = env::var("CHAIN_ID") {
        if cfg.wallet_server.chain_id.is_none() {
            if let Ok(parsed) = v.parse::<u64>() {
                cfg.wallet_server.chain_id = Some(parsed);
            }
        }
    }

    if let Ok(v) = env::var("WALLET_FILE") {
        if cfg.wallet_server.wallet_file.is_none() {
            cfg.wallet_server.wallet_file = Some(PathBuf::from(v));
        }
    }

    if let Ok(v) = env::var("GAS_LIMIT") {
        if cfg.wallet_server.gas_limit.is_none() {
            if let Ok(parsed) = v.parse::<u64>() {
                cfg.wallet_server.gas_limit = Some(parsed);
            }
        }
    }

    if let Ok(v) = env::var("GAS_PRICE") {
        if cfg.wallet_server.gas_price.is_none() {
            if let Ok(parsed) = v.parse::<u128>() {
                cfg.wallet_server.gas_price = Some(parsed);
            }
        }
    }
}

/// Configuration specific to the LLM provider.
#[derive(Deserialize, Debug, Default, PartialEq)]
pub struct LlmConfig {
    /// The Google API key for Gemini.
    pub google_api_key: Option<String>,
    /// The generation configuration for the Gemini model.
    #[serde(rename = "generationConfig")]
    pub generation_config: Option<GenerationConfig>,
}

/// Configuration for Gemini's generation parameters.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    pub temperature: f64,
    pub top_k: u32,
    pub top_p: f64,
    pub max_output_tokens: u32,
    pub stop_sequences: Vec<String>,
}

/// Configuration for tools.
#[derive(Deserialize, Debug, Default, PartialEq)]
pub struct ToolsConfig {
    /// The Google CSE API key for web search tool.
    pub google_search_api_key: Option<String>,
    /// The Google CSE search engine ID (aka `cx`).
    pub google_search_engine_id: Option<String>,
}

/// Configuration for the embedded MCP wallet server.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct WalletServerConfig {
    /// Whether to enable the embedded MCP wallet server.
    pub enable: bool,
    /// The URL of the Ethereum RPC endpoint.
    pub rpc_url: String,
    /// Optional chain ID override to use with the RPC endpoint.
    pub chain_id: Option<u64>,
    /// Optional path to the wallet file managed by mcp-wallet.
    pub wallet_file: Option<PathBuf>,
    /// Optional gas limit to use for transactions.
    pub gas_limit: Option<u64>,
    /// Optional gas price (in wei) to use for transactions.
    pub gas_price: Option<u128>,
    /// The address to bind the MCP server to (kept for compatibility; may be unused
    /// when running in-process/stdio transport).
    pub listen_address: String,
}

impl Default for WalletServerConfig {
    fn default() -> Self {
        Self {
            enable: true,
            rpc_url: "http://127.0.0.1:8545".to_string(),
            chain_id: None,
            wallet_file: None,
            gas_limit: None,
            gas_price: None,
            listen_address: "127.0.0.1:8546".to_string(),
        }
    }
}

/// Loads the application configuration from the default path.
#[allow(dead_code)]
pub fn load() -> Result<Config> {
    let config_path = get_default_config_path()?;
    load_from_path(&config_path)
}

/// Loads the application configuration from a specific path.
///
/// If the file does not exist, a default configuration is returned.
#[allow(dead_code)]
pub fn load_from_path(path: &Path) -> Result<Config> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory at {:?}", parent))?;
        }
        let mut cfg = Config::default();
        apply_env_defaults(&mut cfg);
        return Ok(cfg);
    }

    let config_content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file at {:?}", path))?;

    let config: Config =
        serde_json::from_str(&config_content).with_context(|| "Failed to parse config file")?;
    Ok(config)
}

/// Returns the default path to the configuration file.
#[allow(dead_code)]
fn get_default_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Failed to find user's config directory")?
        .join("eth-partner");

    Ok(config_dir.join("config.json"))
}

#[cfg(test)]
mod tests {
    use super::{
        load_from_path, Config, GenerationConfig, LlmConfig, ToolsConfig, WalletServerConfig,
    };
    use std::fs;
    use tempfile::tempdir;

    struct EnvGuard {
        saved: Vec<(&'static str, Option<String>)>,
    }

    impl EnvGuard {
        fn new(keys: &[&'static str]) -> Self {
            let mut saved = Vec::new();
            for &k in keys {
                let prev = std::env::var(k).ok();
                saved.push((k, prev));
                std::env::remove_var(k);
            }
            Self { saved }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (k, v) in self.saved.drain(..) {
                if let Some(val) = v {
                    std::env::set_var(k, val);
                } else {
                    std::env::remove_var(k);
                }
            }
        }
    }

    #[test]
    fn test_load_config_from_file() {
        let _guard = EnvGuard::new(&[
            "ETH_RPC_URL",
            "CHAIN_ID",
            "WALLET_FILE",
            "GAS_LIMIT",
            "GAS_PRICE",
        ]);
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.json");

        let config_content = r#"
        {
            "llm": {
                "google_api_key": "test_key",
                "generationConfig": {
                    "temperature": 0.8,
                    "topK": 2,
                    "topP": 0.9,
                    "maxOutputTokens": 1024,
                    "stopSequences": ["stop"]
                }
            },
            "wallet_server": {
                "rpc_url": "http://localhost:1234",
                "listen_address": "127.0.0.1:5678"
            }
        }
        "#;

        fs::write(&config_path, config_content).unwrap();

        let config = load_from_path(&config_path).unwrap();

        assert_eq!(
            config,
            Config {
                llm: LlmConfig {
                    google_api_key: Some("test_key".to_string()),
                    generation_config: Some(GenerationConfig {
                        temperature: 0.8,
                        top_k: 2,
                        top_p: 0.9,
                        max_output_tokens: 1024,
                        stop_sequences: vec!["stop".to_string()],
                    })
                },
                tools: ToolsConfig {
                    google_search_api_key: None,
                    google_search_engine_id: None,
                },
                wallet_server: WalletServerConfig {
                    enable: true,
                    rpc_url: "http://localhost:1234".to_string(),
                    chain_id: None,
                    wallet_file: None,
                    gas_limit: None,
                    gas_price: None,
                    listen_address: "127.0.0.1:5678".to_string(),
                },
            }
        );
    }

    #[test]
    fn test_load_config_with_generation_config() {
        let _guard = EnvGuard::new(&[
            "ETH_RPC_URL",
            "CHAIN_ID",
            "WALLET_FILE",
            "GAS_LIMIT",
            "GAS_PRICE",
        ]);
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.json");

        let config_content = r#"
        {
            "llm": {
                "generationConfig": {
                    "temperature": 0.7,
                    "topK": 3,
                    "topP": 0.8,
                    "maxOutputTokens": 512,
                    "stopSequences": []
                }
            }
        }
        "#;

        fs::write(&config_path, config_content).unwrap();

        let config = load_from_path(&config_path).unwrap();

        assert_eq!(
            config.llm.generation_config,
            Some(GenerationConfig {
                temperature: 0.7,
                top_k: 3,
                top_p: 0.8,
                max_output_tokens: 512,
                stop_sequences: vec![],
            })
        );
    }

    #[test]
    fn test_load_default_config_if_not_exists() {
        let _guard = EnvGuard::new(&[
            "ETH_RPC_URL",
            "CHAIN_ID",
            "WALLET_FILE",
            "GAS_LIMIT",
            "GAS_PRICE",
        ]);
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("non_existent_config.json");

        let config = load_from_path(&config_path).unwrap();

        assert_eq!(config, Config::default());
    }
}
