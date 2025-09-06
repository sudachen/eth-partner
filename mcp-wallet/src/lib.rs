//! MCP Wallet Server - A simple Ethereum wallet server with MCP interface
//!
//! This library provides functionality for managing Ethereum accounts, creating and signing
//! transactions, and handling wallet operations through a simple interface.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

/// Handles MCP commands and their execution logic.
pub mod commands;

/// Defines error types and a custom `Result` type for the wallet.
pub mod error;
pub mod models;
pub mod transaction;
pub mod wallet;

// Re-export commonly used types and traits
pub use error::{Result, WalletError};
pub use wallet::Wallet;

/// The prelude module provides a convenient way to import the most common types.
pub mod prelude {
    pub use crate::error::{Result, WalletError};
    pub use crate::models::{Eip1559TransactionRequest, Network, SignedTransaction};
    pub use crate::wallet::Wallet;
}
