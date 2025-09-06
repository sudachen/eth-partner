//! Command handlers for the MCP wallet server

use crate::{
    transaction::TransactionBuilder,
    wallet::Wallet,
    WalletError,
};
use ethers::types::{Address, U256};
use serde::Serialize;
use std::str::FromStr;

/// A serializable representation of an account for JSON responses.
#[derive(Debug, Serialize)]
pub struct JsonAccount<'a> {
    address: String,
    nonce: u64,
    aliases: &'a [String],
}

/// Handles the `new-account` command.
pub fn new_account(wallet: &mut Wallet, alias: Option<String>) -> Result<Address, WalletError> {
    let alias_str = alias.as_deref().unwrap_or_default();
    wallet.create_account(alias_str)
}

/// Handles the `list-accounts` command.
pub fn list_accounts(wallet: &Wallet) -> Result<Vec<JsonAccount>, WalletError> {
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
pub fn set_alias(
    wallet: &mut Wallet,
    address: String,
    alias: String,
) -> Result<(), WalletError> {
    let address = Address::from_str(&address)
        .map_err(|_| WalletError::WalletError(format!("Invalid address: {}", address)))?;
    wallet.add_alias(address, alias)
}

/// Handles the `create-tx` command.
pub fn create_tx(
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
