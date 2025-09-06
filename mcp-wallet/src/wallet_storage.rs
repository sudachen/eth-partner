//! # Wallet Storage
//!
//! This module handles loading and saving of the wallet's private key.

use crate::prelude::*;
use ethers::signers::LocalWallet;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;

/// Represents the structure of the wallet key file.
#[derive(Serialize, Deserialize, Debug)]
struct KeyFile {
    private_key: String,
}

/// Loads a private key from a JSON file.
///
/// The file is expected to be in the format: `{"private_key": "0x..."}`.
///
/// # Arguments
///
/// * `path` - The path to the key file.
///
/// # Returns
///
/// A `Result` containing the `LocalWallet` instance or a `WalletError`.
pub fn load_key(path: &Path) -> Result<LocalWallet> {
    let contents = std::fs::read_to_string(path)?;
    let key_file: KeyFile = serde_json::from_str(&contents)?;
    let wallet = LocalWallet::from_str(&key_file.private_key)
        .map_err(|e| WalletError::InvalidPrivateKey(e.to_string()))?;
    Ok(wallet)
}