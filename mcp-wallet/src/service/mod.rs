//! The MCP service implementation for the wallet.

use crate::{eth_client::EthClient, wallet::Wallet, WalletError};
use ethers::types::{Address, U256};
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

// --- Tool Parameter Structs ---

#[derive(Deserialize, Debug, schemars::JsonSchema)]
struct NewAccountParams {
    alias: Option<String>,
}

#[derive(Deserialize, Debug, schemars::JsonSchema)]
struct SetAliasParams {
    address: String,
    alias: String,
}

#[derive(Deserialize, Debug, schemars::JsonSchema)]
struct CreateTxParams {
    from: String,
    to: String,
    value: String,
    chain_id: u64,
    gas: Option<u64>,
    max_fee_per_gas: Option<String>,
    max_priority_fee_per_gas: Option<String>,
}

#[derive(Deserialize, Debug, schemars::JsonSchema)]
struct SignTxParams {
    from: String,
    tx_json: Value,
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

    #[tool(description = "Creates a new Ethereum account.")]
    async fn new_account(
        &self,
        params: Parameters<NewAccountParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut wallet = self.wallet.lock().await;
        let address = wallet
            .create_account(params.0.alias.as_deref().unwrap_or(""))
            .map_err(to_internal_error)?;
        let result = json!({ "address": format!("0x{:x}", address) });
        Ok(CallToolResult::structured(result))
    }

    #[tool(description = "Lists all Ethereum accounts in the wallet.")]
    async fn list_accounts(&self) -> Result<CallToolResult, ErrorData> {
        let wallet = self.wallet.lock().await;
        let accounts = wallet.list_accounts();
        let json_accounts: Vec<_> = accounts
            .into_iter()
            .map(|(address, account)| {
                json!({ "address": format!("0x{:x}", address), "nonce": account.nonce, "aliases": account.aliases })
            })
            .collect();
        let result = serde_json::to_value(json_accounts).map_err(to_internal_error)?;
        Ok(CallToolResult::structured(result))
    }

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

fn to_invalid_params_error(e: serde_json::Error) -> ErrorData {
    ErrorData::invalid_params(e.to_string(), None)
}
