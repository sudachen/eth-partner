//! # Ethereum RPC Client
//!
//! This module provides a client for interacting with an Ethereum node via RPC.

use crate::prelude::*;
use ethers::{
    core::types::transaction::eip2718::TypedTransaction,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, Bytes, H256, Transaction},
    utils::format_ether,
};
use std::str::FromStr;

/// A client for interacting with an Ethereum RPC endpoint.
#[derive(Debug)]
pub struct EthClient {
    /// The Ethers provider for making RPC calls.
    provider: Provider<Http>,
    /// The wallet used for signing transactions.
    signer: Option<LocalWallet>,
}

impl EthClient {
    /// Creates a new Ethereum RPC client.
    ///
    /// # Arguments
    ///
    /// * `rpc_url` - The URL of the Ethereum RPC endpoint.
    /// * `signer` - An optional wallet for signing transactions.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `EthClient` or a `WalletError` if the
    /// client could not be created.
    pub fn new(rpc_url: &str, signer: Option<LocalWallet>) -> Result<Self> {
        let http_provider = Http::from_str(rpc_url)
            .map_err(|e| WalletError::RpcClientInitialization(e.to_string()))?;
        let provider = Provider::new(http_provider);
        Ok(Self { provider, signer })
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

    /// Transfers ETH to a specified address.
    ///
    /// This method creates, signs, and sends a transaction.
    ///
    /// # Arguments
    ///
    /// * `to_address` - The recipient's Ethereum address.
    /// * `amount_eth` - The amount of ETH to send.
    ///
    /// # Returns
    ///
    /// A `Result` containing the transaction hash (`H256`) or a `WalletError`.
    pub async fn transfer_eth(&self, to_address: &str, amount_eth: f64) -> Result<H256> {
        let signer = self
            .signer
            .as_ref()
            .ok_or_else(|| WalletError::WalletError("No signer available".to_string()))?;

        let to_addr = Address::from_str(to_address)
            .map_err(|e| WalletError::InvalidPrivateKey(e.to_string()))?;

        let amount_wei = ethers::utils::parse_ether(amount_eth)?;

        let tx_request = TypedTransaction::Eip1559(ethers::types::Eip1559TransactionRequest {
            to: Some(to_addr.into()),
            from: Some(signer.address()),
            value: Some(amount_wei),
            ..Default::default()
        });

        let signed_tx = signer.sign_transaction(&tx_request).await?;
        let rlp_signed = tx_request.rlp_signed(&signed_tx);

        let pending_tx = self.provider.send_raw_transaction(rlp_signed).await?;

        Ok(*pending_tx)
    }
}