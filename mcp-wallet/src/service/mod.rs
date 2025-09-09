//! The MCP service implementation for the wallet.

use crate::{eth_client::EthClient, wallet::Wallet, WalletError};
use ethers::types::{Address, H256, U256};
use ethers::utils::to_checksum;
use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ErrorData},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ServerHandler,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::str::FromStr;
use std::sync::Arc;

use tokio::sync::Mutex;

/// Normalizes a private key string.
///
/// - Trims whitespace
/// - Strips optional `0x` prefix
/// - Validates it is exactly 64 hex chars and not all zeros
/// - Returns lowercase hex string on success
fn normalize_private_key_hex(input: &str) -> Option<String> {
    let pk = input.trim();
    let normalized = pk.strip_prefix("0x").unwrap_or(pk);
    if normalized.len() != 64 {
        return None;
    }
    if !normalized.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    if !normalized.chars().any(|c| c != '0') {
        return None;
    }
    Some(normalized.to_ascii_lowercase())
}

// --- Tool Parameter Structs ---

/// Parameters for the `new_account` tool.
#[derive(Deserialize, Debug, schemars::JsonSchema)]
struct NewAccountParams {
    /// An optional alias for the new account.
    alias: Option<String>,
}

/// Parameters for the `set_alias` tool.
#[derive(Deserialize, Debug, schemars::JsonSchema)]
struct SetAliasParams {
    /// The Ethereum address to set an alias for.
    address: String,
    /// The alias to set for the address.
    alias: String,
}

/// Parameters for the `import_private_key` tool.
#[derive(Deserialize, Debug, schemars::JsonSchema)]
struct ImportPrivateKeyParams {
    /// The private key in hex format (0x-prefixed or raw 64 hex chars).
    private_key: String,
}

/// Parameters for the `create_tx` tool.
#[derive(Deserialize, Debug, schemars::JsonSchema)]
struct CreateTxParams {
    /// The identifier (address or alias) of the account to send from.
    from: String,
    /// The recipient's Ethereum address.
    to: String,
    /// The amount of ETH to send.
    value: String,
    /// The chain ID for the transaction.
    chain_id: u64,
    /// The gas limit for the transaction.
    gas: Option<u64>,
    /// The maximum fee per gas for the transaction.
    max_fee_per_gas: Option<String>,
    /// The maximum priority fee per gas for the transaction.
    max_priority_fee_per_gas: Option<String>,
}

/// Parameters for the `sign_tx` tool.
#[derive(Deserialize, Debug, schemars::JsonSchema)]
struct SignTxParams {
    /// The identifier (address or alias) of the account to sign with.
    from: String,
    /// The transaction to sign.
    tx_json: Value,
}

/// Parameters for the `eth_getBalance` tool.
#[derive(Deserialize, Debug, schemars::JsonSchema)]
struct GetBalanceParams {
    /// The Ethereum address to query.
    address: String,
}

/// Parameters for the `eth_sendSignedTransaction` tool.
#[derive(Deserialize, Debug, schemars::JsonSchema)]
struct SendSignedTxParams {
    /// The raw, signed transaction as a hex-encoded string.
    signed_transaction_hex: String,
}

/// Parameters for the `eth_getTransactionInfo` tool.
#[derive(Deserialize, Debug, schemars::JsonSchema)]
struct GetTxInfoParams {
    /// The transaction hash as a hex-encoded string.
    transaction_hash: String,
}

/// Parameters for the `eth_getTransactionReceipt` tool.
#[derive(Deserialize, Debug, schemars::JsonSchema)]
struct GetTxReceiptParams {
    /// The transaction hash as a hex-encoded string.
    transaction_hash: String,
}

/// Parameters for the `eth_transferEth` tool.
#[derive(Deserialize, Debug, schemars::JsonSchema)]
struct TransferEthParams {
    /// The identifier (address or alias) of the account to send from.
    from: String,
    /// The recipient's Ethereum address.
    to: String,
    /// The amount of ETH to send.
    value_eth: f64,
    /// The chain ID for the transaction.
    chain_id: u64,
}

/// The service handler for the wallet.
#[derive(Clone)]
pub struct WalletHandler {
    tool_router: ToolRouter<Self>,
    wallet: Arc<Mutex<Wallet>>,
    eth_client: Arc<EthClient>,
}

#[tool_router]
#[allow(missing_docs)]
impl WalletHandler {
    /// Creates a new `WalletHandler`.
    pub fn new(wallet: Arc<Mutex<Wallet>>, eth_client: Arc<EthClient>) -> Self {
        Self {
            wallet,
            eth_client,
            tool_router: Self::tool_router(),
        }
    }

    /// Creates a new Ethereum account.
    #[tool(description = "Creates a new Ethereum account.")]
    async fn new_account(
        &self,
        params: Parameters<NewAccountParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut wallet = self.wallet.lock().await;
        let address = wallet
            .create_account(params.0.alias.as_deref().unwrap_or(""))
            .map_err(to_internal_error)?;
        let result = json!({ "address": to_checksum(&address, None) });
        Ok(CallToolResult::structured(result))
    }

    /// Gets a transaction receipt by its hash.
    #[tool(description = "Gets a transaction receipt by its hash.")]
    async fn eth_get_transaction_receipt(
        &self,
        params: Parameters<GetTxReceiptParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let tx_hash = H256::from_str(
            params
                .0
                .transaction_hash
                .strip_prefix("0x")
                .unwrap_or(&params.0.transaction_hash),
        )
        .map_err(|e| to_invalid_params_error(e.to_string()))?;

        let receipt_opt = self
            .eth_client
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(to_internal_error)?;

        let result = if let Some(rcpt) = receipt_opt {
            let status_num = rcpt.status.map(|s| s.as_u64()).unwrap_or(0);
            let status_text = if status_num == 1 { "success" } else { "failed" };
            json!({
                "found": true,
                "status": status_text,
                "block_number": rcpt.block_number.map(|b| b.as_u64()),
                "transaction_hash": format!("0x{:x}", rcpt.transaction_hash),
            })
        } else {
            json!({ "found": false, "status": "pending" })
        };

        Ok(CallToolResult::structured(result))
    }

    /// Lists all Ethereum accounts in the wallet.
    #[tool(description = "Lists all Ethereum accounts in the wallet.")]
    async fn list_accounts(&self) -> Result<CallToolResult, ErrorData> {
        let wallet = self.wallet.lock().await;
        let accounts = wallet.list_accounts();
        let json_accounts: Vec<_> = accounts
            .into_iter()
            .map(|(address, account)| {
                json!({
                    "address": to_checksum(&address, None),
                    "nonce": account.nonce,
                    "aliases": account.aliases,
                    "is_signing": account.private_key.is_some()
                })
            })
            .collect();
        let result = serde_json::to_value(json_accounts).map_err(to_internal_error)?;
        Ok(CallToolResult::structured(result))
    }

    /// Sets an alias for an Ethereum account.
    #[tool(description = "Sets an alias for an Ethereum account.")]
    async fn set_alias(
        &self,
        params: Parameters<SetAliasParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut wallet = self.wallet.lock().await;
        let address = Address::from_str(&params.0.address)
            .map_err(|_| to_internal_error(format!("Invalid address: {}", params.0.address)))?;
        wallet
            .add_alias(address, params.0.alias.clone())
            .map_err(to_internal_error)?;
        let result = Value::Null;
        Ok(CallToolResult::structured(result))
    }

    /// Imports a private key, creating or upgrading an account as needed.
    #[tool(description = "Imports a private key, creating or upgrading an account as needed.")]
    async fn import_private_key(
        &self,
        params: Parameters<ImportPrivateKeyParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut wallet = self.wallet.lock().await;
        let normalized = normalize_private_key_hex(&params.0.private_key).ok_or_else(|| {
            to_invalid_params_error("Invalid private key format (expect 32-byte hex)".to_string())
        })?;
        let address = wallet
            .import_private_key(&normalized, "")
            .map_err(to_internal_error)?;
        let result = json!({ "address": to_checksum(&address, None) });
        Ok(CallToolResult::structured(result))
    }

    /// Creates an EIP-1559 transaction request.
    #[tool(description = "Creates an EIP-1559 transaction request.")]
    async fn create_tx(
        &self,
        params: Parameters<CreateTxParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let wallet = self.wallet.lock().await;
        let (from_account, _) = wallet
            .get_account(&params.0.from)
            .ok_or_else(|| to_internal_error(WalletError::SignerNotFound(params.0.from.clone())))?;
        let to_address = Address::from_str(&params.0.to)
            .map_err(|_| to_internal_error(format!("Invalid 'to' address: {}", params.0.to)))?;
        let value = U256::from_dec_str(&params.0.value)
            .map_err(|_| to_internal_error(format!("Invalid 'value': {}", params.0.value)))?;

        let mut builder = crate::transaction::TransactionBuilder::new()
            .chain_id(params.0.chain_id)
            .to(to_address)
            .value(value)
            .nonce(from_account.nonce);

        if let Some(gas) = params.0.gas {
            builder = builder.gas(gas);
        }
        if let Some(max_fee_str) = &params.0.max_fee_per_gas {
            let max_fee = U256::from_dec_str(max_fee_str).map_err(|_| {
                to_internal_error(format!("Invalid 'max_fee_per_gas': {}", max_fee_str))
            })?;
            builder = builder.max_fee_per_gas(max_fee);
        }
        if let Some(max_prio_str) = &params.0.max_priority_fee_per_gas {
            let max_prio = U256::from_dec_str(max_prio_str).map_err(|_| {
                to_internal_error(format!(
                    "Invalid 'max_priority_fee_per_gas': {}",
                    max_prio_str
                ))
            })?;
            builder = builder.max_priority_fee_per_gas(max_prio);
        }

        let tx_request = builder.build();
        let result = serde_json::to_value(&tx_request).map_err(to_internal_error)?;
        Ok(CallToolResult::structured(result))
    }

    /// Signs a transaction with a specified account.
    #[tool(description = "Signs a transaction with a specified account.")]
    async fn sign_tx(&self, params: Parameters<SignTxParams>) -> Result<CallToolResult, ErrorData> {
        let mut wallet = self.wallet.lock().await;
        let tx_request: crate::models::Eip1559TransactionRequest =
            serde_json::from_value(params.0.tx_json.clone()).map_err(to_invalid_params_error)?;
        let signed_tx = wallet
            .sign_transaction(&tx_request, &params.0.from)
            .await
            .map_err(to_internal_error)?;
        let result = serde_json::to_value(JsonSignedTransaction::from(signed_tx))
            .map_err(to_internal_error)?;
        Ok(CallToolResult::structured(result))
    }

    /// Gets the current block number of the Ethereum network.
    #[tool(description = "Gets the current block number of the Ethereum network.")]
    async fn eth_get_current_block(&self) -> Result<CallToolResult, ErrorData> {
        let block_number = self
            .eth_client
            .get_current_block()
            .await
            .map_err(to_internal_error)?;
        let result = json!({ "block_number": block_number });
        Ok(CallToolResult::structured(result))
    }

    /// Gets the ETH balance for a given address.
    #[tool(description = "Gets the ETH balance for a given address.")]
    async fn eth_get_balance(
        &self,
        params: Parameters<GetBalanceParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let balance = self
            .eth_client
            .get_balance(&params.0.address)
            .await
            .map_err(to_internal_error)?;
        let result = json!({ "balance_eth": balance });
        Ok(CallToolResult::structured(result))
    }

    /// Sends a signed transaction to the network.
    #[tool(description = "Sends a signed transaction to the network.")]
    async fn eth_send_signed_transaction(
        &self,
        params: Parameters<SendSignedTxParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let tx_hash = self
            .eth_client
            .send_signed_transaction(&params.0.signed_transaction_hex)
            .await
            .map_err(to_internal_error)?;
        let result = json!({ "transaction_hash": format!("0x{:x}", tx_hash) });
        Ok(CallToolResult::structured(result))
    }

    /// Gets information about a transaction by its hash.
    #[tool(description = "Gets information about a transaction by its hash.")]
    async fn eth_get_transaction_info(
        &self,
        params: Parameters<GetTxInfoParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let tx_hash = H256::from_str(
            params
                .0
                .transaction_hash
                .strip_prefix("0x")
                .unwrap_or(&params.0.transaction_hash),
        )
        .map_err(|e| to_invalid_params_error(e.to_string()))?;

        let tx_info = self
            .eth_client
            .get_transaction_info(tx_hash)
            .await
            .map_err(to_internal_error)?;

        let result = serde_json::to_value(tx_info).map_err(|e| to_internal_error(e.to_string()))?;
        Ok(CallToolResult::structured(result))
    }

    /// Creates, signs, and sends an ETH transfer transaction.
    #[tool(description = "Creates, signs, and sends an ETH transfer transaction.")]
    async fn eth_transfer_eth(
        &self,
        params: Parameters<TransferEthParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut wallet = self.wallet.lock().await;

        let to_address = Address::from_str(&params.0.to).map_err(|_| {
            to_invalid_params_error(format!("Invalid 'to' address: {}", params.0.to))
        })?;
        let value_wei = ethers::utils::parse_ether(params.0.value_eth)
            .map_err(|e| to_invalid_params_error(e.to_string()))?;

        // Create the transaction request
        let (from_account, _) = wallet
            .get_account(&params.0.from)
            .ok_or_else(|| to_internal_error(WalletError::SignerNotFound(params.0.from.clone())))?;

        let tx_request = crate::models::Eip1559TransactionRequest {
            to: Some(to_address),
            value: value_wei,
            chain_id: params.0.chain_id,
            nonce: from_account.nonce.into(),
            ..Default::default()
        };

        // Sign the transaction
        let signed_tx = wallet
            .sign_transaction(&tx_request, &params.0.from)
            .await
            .map_err(to_internal_error)?;

        // Send the transaction
        let raw_tx_hex = format!("0x{}", hex::encode(signed_tx.raw_transaction));
        let tx_hash = self
            .eth_client
            .send_signed_transaction(&raw_tx_hex)
            .await
            .map_err(to_internal_error)?;

        let result = json!({ "transaction_hash": format!("0x{:x}", tx_hash) });
        Ok(CallToolResult::structured(result))
    }
}

#[tool_handler]
impl ServerHandler for WalletHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A wallet".into()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_logging()
                .build(),
            ..Default::default()
        }
    }
}

// --- Helpers ---

#[derive(Serialize)]
struct JsonSignedTransaction {
    raw_transaction: String,
    hash: String,
    signature: (u64, String, String),
    chain_id: u64,
}

impl From<crate::models::SignedTransaction> for JsonSignedTransaction {
    fn from(tx: crate::models::SignedTransaction) -> Self {
        Self {
            raw_transaction: format!("0x{}", hex::encode(tx.raw_transaction)),
            hash: format!("0x{}", hex::encode(tx.hash)),
            signature: (
                tx.signature.0,
                format!("0x{}", hex::encode(tx.signature.1)),
                format!("0x{}", hex::encode(tx.signature.2)),
            ),
            chain_id: tx.chain_id,
        }
    }
}

fn to_internal_error<E: std::fmt::Display>(e: E) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

fn to_invalid_params_error<E: std::fmt::Display>(e: E) -> ErrorData {
    ErrorData::invalid_params(e.to_string(), None)
}
