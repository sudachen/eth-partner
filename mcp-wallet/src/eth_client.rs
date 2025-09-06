//! # Ethereum RPC Client
//!
//! This module provides a client for interacting with an Ethereum node via RPC.

use crate::prelude::*;
use ethers::{
    providers::{Http, Middleware, Provider},
    types::{Address, Bytes, H256, Transaction},
    utils::format_ether,
};
use std::str::FromStr;

/// A client for interacting with an Ethereum RPC endpoint.
#[derive(Debug)]
pub struct EthClient {
    /// The Ethers provider for making RPC calls.
    provider: Provider<Http>,
}

impl EthClient {
    /// Creates a new Ethereum RPC client.
    ///
    /// # Arguments
    ///
    /// * `rpc_url` - The URL of the Ethereum RPC endpoint.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `EthClient` or a `WalletError` if the
    /// client could not be created.
    pub fn new(rpc_url: &str) -> Result<Self> {
        let http_provider = Http::from_str(rpc_url)
            .map_err(|e| WalletError::RpcClientInitialization(e.to_string()))?;
        let provider = Provider::new(http_provider);
        Ok(Self { provider })
    }

    /// Gets the current block number from the Ethereum network.
    ///
    /// # Returns
    ///
    /// A `Result` containing the current block number (`u64`) or a `WalletError`.
    pub async fn get_current_block(&self) -> Result<u64> {
        let block_number = self.provider.get_block_number().await?;
        Ok(block_number.as_u64())
    }

    /// Gets the balance of a given Ethereum address.
    ///
    /// # Arguments
    ///
    /// * `address` - The Ethereum address to query.
    ///
    /// # Returns
    ///
    /// A `Result` containing the balance in Ether (as a `String`) or a `WalletError`.
    pub async fn get_balance(&self, address: &str) -> Result<String> {
        let addr = Address::from_str(address)
            .map_err(|e| WalletError::InvalidPrivateKey(e.to_string()))?;
        let balance_wei = self.provider.get_balance(addr, None).await?;
        Ok(format_ether(balance_wei))
    }

    /// Sends a signed transaction to the Ethereum network.
    ///
    /// # Arguments
    ///
    /// * `signed_tx_hex` - The raw, signed transaction as a hex-encoded string.
    ///
    /// # Returns
    ///
    /// A `Result` containing the transaction hash (`H256`) or a `WalletError`.
    pub async fn send_signed_transaction(&self, signed_tx_hex: &str) -> Result<H256> {
        let tx_bytes = hex::decode(signed_tx_hex.strip_prefix("0x").unwrap_or(signed_tx_hex))?;
        let tx_bytes = Bytes::from(tx_bytes);

        let pending_tx = self.provider.send_raw_transaction(tx_bytes).await?;
        Ok(*pending_tx)
    }

    /// Gets information about a transaction by its hash.
    ///
    /// # Arguments
    ///
    /// * `tx_hash` - The hash of the transaction to query.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<Transaction>` or a `WalletError`.
    /// The option will be `None` if the transaction is not found.
    pub async fn get_transaction_info(&self, tx_hash: H256) -> Result<Option<Transaction>> {
        let tx_info = self.provider.get_transaction(tx_hash).await?;
        Ok(tx_info)
    }
}