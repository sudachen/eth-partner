//! Data models for the MCP Wallet

pub mod network;
pub mod transaction;

pub use self::network::Network;
pub use self::transaction::{Eip1559TransactionRequest, SignedTransaction};

use serde::{Deserialize, Serialize};

/// Represents a transaction creation request from the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    /// Sender's address or alias
    pub from: String,
    /// Recipient's address
    pub to: String,
    /// Amount to send in wei
    pub value: String,
    /// Transaction nonce (optional, will use current nonce if not provided)
    pub nonce: Option<u64>,
    /// Gas limit (optional)
    pub gas: Option<u64>,
    /// Gas price in wei (optional)
    pub gas_price: Option<String>,
    /// Max fee per gas (EIP-1559, optional)
    pub max_fee_per_gas: Option<String>,
    /// Max priority fee per gas (EIP-1559, optional)
    pub max_priority_fee_per_gas: Option<String>,
    /// Chain ID (optional)
    pub chain_id: Option<u64>,
}

/// Represents a wallet account with its associated data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Private key in hex format
    pub private_key: String,
    /// Public key in hex format
    pub public_key: String,
    /// Current nonce for the account
    pub nonce: u64,
    /// List of aliases associated with this account
    pub aliases: Vec<String>,
}

// Tests moved to tests/model_tests.rs
