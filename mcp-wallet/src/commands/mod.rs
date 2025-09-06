//! MCP command processing for the wallet server.

pub mod tool_definition;

use crate::{transaction::TransactionBuilder, wallet::Wallet, WalletError, models::Eip1559TransactionRequest};
use self::tool_definition::generate_tool_definition;
use ethers::types::{Address, U256};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;

// --- MCP Request/Response Structs ---

/// Represents an incoming MCP command from stdin.
#[derive(Debug, Deserialize)]
pub struct McpRequest {
    /// The command to execute (e.g., "new-account", "sign-tx").
    pub command: String,
    /// The parameters for the command, as a JSON value.
    #[serde(default)]
    pub params: Value,
}

/// Represents a response to an MCP command, sent to stdout.
#[derive(Debug, Serialize)]
pub struct McpResponse {
    /// The status of the command ("success" or "error").
    pub status: String,
    /// The data returned by a successful command, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    /// The error message if the command failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl McpResponse {
    /// Creates a success response with data.
    pub fn success(data: Value) -> Self {
        Self {
            status: "success".to_string(),
            data: Some(data),
            error: None,
        }
    }

    /// Creates an error response with a message.
    pub fn error(message: String) -> Self {
        Self {
            status: "error".to_string(),
            data: None,
            error: Some(message),
        }
    }
}

// --- Command Dispatch ---

/// Parses and handles a raw JSON command string.
pub async fn handle_mcp_command(command: &str, wallet: &mut Wallet) -> McpResponse {
    match serde_json::from_str::<McpRequest>(command) {
        Ok(request) => dispatch_command(request, wallet).await,
        Err(e) => McpResponse::error(format!("Invalid JSON request: {}", e)),
    }
}

/// Dispatches a parsed command to the appropriate handler.
async fn dispatch_command(request: McpRequest, wallet: &mut Wallet) -> McpResponse {
    let result = match request.command.as_str() {
        "get_tool_definition" => {
            let definition = generate_tool_definition();
            serde_json::to_value(definition).map_err(WalletError::JsonError)
        }
        "new-account" => {
            let params: NewAccountParams = serde_json::from_value(request.params).unwrap_or_default();
            new_account(wallet, params.alias)
                .map(|address| serde_json::json!({ "address": format!("0x{:x}", address) }))
        }
        "list-accounts" => list_accounts(wallet).map(|accounts| serde_json::json!(accounts)),
        "set-alias" => {
            let params: SetAliasParams = match serde_json::from_value(request.params) {
                Ok(p) => p,
                Err(e) => return McpResponse::error(format!("Invalid params for set-alias: {}", e)),
            };
            set_alias(wallet, params.address, params.alias).map(|_| Value::Null)
        }
        "create-tx" => {
            let params: CreateTxParams = match serde_json::from_value(request.params) {
                Ok(p) => p,
                Err(e) => return McpResponse::error(format!("Invalid params for create-tx: {}", e)),
            };
            create_tx(
                wallet,
                params.from,
                params.to,
                params.value,
                params.chain_id,
                params.gas,
                params.max_fee_per_gas,
                params.max_priority_fee_per_gas,
            )
        }
        "sign-tx" => {
            let params: SignTxParams = match serde_json::from_value(request.params) {
                Ok(p) => p,
                Err(e) => return McpResponse::error(format!("Invalid params for sign-tx: {}", e)),
            };
            sign_tx(wallet, &params.tx_json, &params.from).await
        }
        _ => Err(WalletError::WalletError(format!(
            "Unknown command: {}",
            request.command
        ))),
    };

    match result {
        Ok(data) => McpResponse::success(data),
        Err(e) => McpResponse::error(e.to_string()),
    }
}

// --- Command Handlers ---

/// A serializable representation of an account for JSON responses.
#[derive(Debug, Serialize)]
struct JsonAccount<'a> {
    address: String,
    nonce: u64,
    aliases: &'a [String],
}

/// Handles the `new-account` command.
fn new_account(wallet: &mut Wallet, alias: Option<String>) -> Result<Address, WalletError> {
    let alias_str = alias.as_deref().unwrap_or_default();
    wallet.create_account(alias_str)
}

/// Handles the `list-accounts` command.
fn list_accounts(wallet: &Wallet) -> Result<Vec<JsonAccount<'_>>, WalletError> {
    let accounts = wallet
        .list_accounts()
        .into_iter()
        .map(|(address, account)| JsonAccount {
            address: format!("0x{:x}", address),
            nonce: account.nonce,
            aliases: &account.aliases,
        })
        .collect();
    Ok(accounts)
}

/// Handles the `set-alias` command.
fn set_alias(
    wallet: &mut Wallet,
    address: String,
    alias: String,
) -> Result<(), WalletError> {
    let address = Address::from_str(&address)
        .map_err(|_| WalletError::WalletError(format!("Invalid address: {}", address)))?;
    wallet.add_alias(address, alias)
}

/// Handles the `create-tx` command.
fn create_tx(
    wallet: &Wallet,
    from: String,
    to: String,
    value: String,
    chain_id: u64,
    gas: Option<u64>,
    max_fee_per_gas: Option<String>,
    max_priority_fee_per_gas: Option<String>,
) -> Result<serde_json::Value, WalletError> {
    let (from_account, _) = wallet
        .get_account(&from)
        .ok_or_else(|| WalletError::SignerNotFound(from.clone()))?;

    let to_address = Address::from_str(&to)
        .map_err(|_| WalletError::WalletError(format!("Invalid 'to' address: {}", to)))?;
    let value = U256::from_dec_str(&value)
        .map_err(|_| WalletError::WalletError(format!("Invalid 'value': {}", value)))?;

    let mut builder = TransactionBuilder::new()
        .chain_id(chain_id)
        .to(to_address)
        .value(value)
        .nonce(from_account.nonce);

    if let Some(gas) = gas {
        builder = builder.gas(gas);
    }

    if let Some(max_fee_str) = max_fee_per_gas {
        let max_fee = U256::from_dec_str(&max_fee_str)
            .map_err(|_| WalletError::WalletError(format!("Invalid 'max_fee_per_gas': {}", max_fee_str)))?;
        builder = builder.max_fee_per_gas(max_fee);
    }

    if let Some(max_prio_str) = max_priority_fee_per_gas {
        let max_prio = U256::from_dec_str(&max_prio_str)
            .map_err(|_| WalletError::WalletError(format!("Invalid 'max_priority_fee_per_gas': {}", max_prio_str)))?;
        builder = builder.max_priority_fee_per_gas(max_prio);
    }

    let tx_request = builder.build();
    let tx_json = serde_json::to_value(&tx_request)
        .map_err(|e| WalletError::JsonError(e))?;

    Ok(tx_json)
}

/// Handles the `sign-tx` command.
async fn sign_tx(
    wallet: &mut Wallet,
    tx_json: &Value,
    from: &str,
) -> Result<Value, WalletError> {
    let tx_request: Eip1559TransactionRequest = serde_json::from_value(tx_json.clone())
        .map_err(|e| WalletError::JsonError(e))?;

    let signed_tx = wallet.sign_transaction(&tx_request, from).await?;

    // Create a serializable version with hex strings
    #[derive(Serialize)]
    struct JsonSignedTransaction {
        raw_transaction: String,
        hash: String,
        signature: (u64, String, String),
        chain_id: u64,
    }

    let response_data = JsonSignedTransaction {
        raw_transaction: format!("0x{}", hex::encode(signed_tx.raw_transaction)),
        hash: format!("0x{}", hex::encode(signed_tx.hash)),
        signature: (
            signed_tx.signature.0,
            format!("0x{}", hex::encode(signed_tx.signature.1)),
            format!("0x{}", hex::encode(signed_tx.signature.2)),
        ),
        chain_id: signed_tx.chain_id,
    };

    let response = serde_json::to_value(&response_data)
        .map_err(|e| WalletError::JsonError(e))?;

    Ok(response)
}

// --- Parameter Structs ---

#[derive(Debug, Deserialize, Default)]
struct NewAccountParams {
    alias: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SetAliasParams {
    address: String,
    alias: String,
}

#[derive(Debug, Deserialize)]
struct CreateTxParams {
    from: String,
    to: String,
    value: String,
    chain_id: u64,
    gas: Option<u64>,
    max_fee_per_gas: Option<String>,
    max_priority_fee_per_gas: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SignTxParams {
    tx_json: Value,
    from: String,
}
