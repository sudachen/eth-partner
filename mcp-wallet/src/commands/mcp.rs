//! MCP command processing for the wallet server.

use crate::{commands::handlers, wallet::Wallet, WalletError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents an incoming MCP command from stdin.
#[derive(Debug, Deserialize)]
pub struct McpRequest {
    pub command: String,
    #[serde(default)]
    pub params: Value,
}

/// Represents a response to an MCP command, sent to stdout.
#[derive(Debug, Serialize)]
pub struct McpResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
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
        "new-account" => {
            let params: NewAccountParams = serde_json::from_value(request.params).unwrap_or_default();
            handlers::new_account(wallet, params.alias)
                .map(|address| serde_json::json!({ "address": format!("0x{:x}", address) }))
        }
        "list-accounts" => handlers::list_accounts(wallet).map(|accounts| serde_json::json!(accounts)),
        "set-alias" => {
            let params: SetAliasParams = match serde_json::from_value(request.params) {
                Ok(p) => p,
                Err(e) => return McpResponse::error(format!("Invalid params for set-alias: {}", e)),
            };
            handlers::set_alias(wallet, params.address, params.alias).map(|_| Value::Null)
        }
        "create-tx" => {
            let params: CreateTxParams = match serde_json::from_value(request.params) {
                Ok(p) => p,
                Err(e) => return McpResponse::error(format!("Invalid params for create-tx: {}", e)),
            };
            handlers::create_tx(
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
            // Placeholder for sign-tx
            Ok(serde_json::json!("sign-tx is not yet implemented"))
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

// Parameter structs for deserialization

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
