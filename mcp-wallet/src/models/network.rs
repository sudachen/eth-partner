//! Network-related data structures and functionality

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents an Ethereum network
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    /// Ethereum Mainnet
    Mainnet,
    /// Sepolia testnet
    Sepolia,
    /// Goerli testnet
    Goerli,
    /// Local development network (e.g., Hardhat, Anvil)
    Local,
    /// A generic development network
    Devnet,
    /// A custom network where chain ID and RPC URL are provided by the user
    Custom,
}

impl Default for Network {
    fn default() -> Self {
        Network::Mainnet
    }
}

impl FromStr for Network {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(Network::Mainnet),
            "sepolia" => Ok(Network::Sepolia),
            "goerli" => Ok(Network::Goerli),
            "local" => Ok(Network::Local),
            "devnet" => Ok(Network::Devnet),
            "custom" => Ok(Network::Custom),
            _ => Err(format!("'{}' is not a valid network", s)),
        }
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl Network {
    /// Get the chain ID for the network
    pub fn chain_id(&self) -> u64 {
        match self {
            Network::Mainnet => 1,
            Network::Sepolia => 11155111,
            Network::Goerli => 5,
            Network::Local => 1337,
            Network::Devnet => 1337,
            Network::Custom => 0, // Should be provided by the user
        }
    }

    /// Get the default RPC URL for the network
    pub fn default_rpc_url(&self) -> &str {
        match self {
            Network::Mainnet => "https://mainnet.infura.io/v3/",
            Network::Sepolia => "https://sepolia.infura.io/v3/",
            Network::Goerli => "https://goerli.infura.io/v3/",
            Network::Local => "http://localhost:8545",
            Network::Devnet => "http://localhost:8545",
            Network::Custom => "", // Should be provided by the user
        }
    }
}
