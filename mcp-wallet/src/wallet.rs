//! Wallet module for managing Ethereum accounts and aliases

use crate::{
    error::{Result, WalletError},
    models::{Eip1559TransactionRequest, SignedTransaction},
};
use ethers::{
    core::types::{transaction::eip2718::TypedTransaction, U256},
    signers::{LocalWallet, Signer},
    types::Address,
};
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Represents a wallet account with its associated data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Private key in hex format.
    pub private_key: String,
    /// The next nonce to be used for a transaction.
    pub nonce: u64,
    /// List of aliases associated with this account.
    pub aliases: Vec<String>,
}

impl Account {
    /// Creates a new account from a private key.
    pub fn new(private_key: String) -> Self {
        Self {
            private_key,
            nonce: 0,
            aliases: Vec::new(),
        }
    }
}

/// Main wallet structure containing all accounts and aliases.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Wallet {
    /// Map of account addresses to their data.
    accounts: HashMap<Address, Account>,
    /// Map of aliases to account addresses.
    aliases: HashMap<String, Address>,
    /// The primary signer for the wallet, loaded from a key file.
    #[serde(skip)]
    signer: Option<LocalWallet>,
    /// Path to the wallet file.
    #[serde(skip)]
    file_path: Option<PathBuf>,
    /// Whether the wallet has unsaved changes.
    #[serde(skip)]
    dirty: bool,
}

impl Wallet {
    /// Creates a new empty wallet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the primary signer for the wallet.
    pub fn set_signer(&mut self, signer: LocalWallet) {
        self.signer = Some(signer);
    }

    /// Gets the signer for an account by its address.
    pub fn get_signer(&self, address: &Address) -> Result<LocalWallet, WalletError> {
        // First, try to get the signer from the multi-account map
        if let Some(signer) = self
            .accounts
            .get(address)
            .and_then(|acc| LocalWallet::from_str(&acc.private_key).ok())
        {
            return Ok(signer);
        }

        // If not found, check if the global signer matches the requested address
        if let Some(ref signer) = self.signer {
            if signer.address() == *address {
                return Ok(signer.clone());
            }
        }

        // If no matching signer is found, return an error
        Err(WalletError::AccountNotFound(*address))
    }

    /// Creates a new account with a random private key and adds it to the wallet.
    ///
    /// Returns the address of the new account.
    pub fn create_account(&mut self, alias: &str) -> Result<Address> {
        let wallet = LocalWallet::new(&mut thread_rng());
        self.add_account(wallet, alias)
    }

    /// Imports an account from a private key string.
    pub fn import_private_key(&mut self, private_key: &str, alias: &str) -> Result<Address> {
        let wallet = private_key
            .parse::<LocalWallet>()
            .map_err(|e| WalletError::InvalidPrivateKey(e.to_string()))?;
        self.add_account(wallet, alias)
    }

    /// Adds an account to the wallet.
    fn add_account(&mut self, wallet: LocalWallet, alias: &str) -> Result<Address> {
        let address = wallet.address();
        if self.accounts.contains_key(&address) {
            return Err(WalletError::AccountAlreadyExists(address));
        }

        let private_key = hex::encode(wallet.signer().to_bytes());
        let mut account = Account::new(private_key);

        if !alias.is_empty() {
            self.add_alias_to_account(&mut account, alias, address)?;
        }

        self.accounts.insert(address, account);
        self.mark_dirty();
        Ok(address)
    }

    /// Adds an alias for an account.
    pub fn add_alias(&mut self, address: Address, alias: String) -> Result<()> {
        if !is_valid_alias(&alias) {
            return Err(WalletError::InvalidAlias(alias));
        }

        if self.aliases.contains_key(&alias) {
            return Err(WalletError::AliasAlreadyExists(alias));
        }

        if let Some(account) = self.accounts.get_mut(&address) {
            self.aliases.insert(alias.clone(), address);
            account.aliases.push(alias);
            self.mark_dirty();
            Ok(())
        } else {
            Err(WalletError::AccountNotFound(address))
        }
    }

    /// Helper to add an alias to an account and the wallet's alias map.
    fn add_alias_to_account(
        &mut self,
        account: &mut Account,
        alias: &str,
        address: Address,
    ) -> Result<()> {
        if !is_valid_alias(alias) {
            return Err(WalletError::InvalidAlias(alias.to_string()));
        }

        if self.aliases.contains_key(alias) {
            return Err(WalletError::AliasAlreadyExists(alias.to_string()));
        }

        let alias_string = alias.to_string();
        self.aliases.insert(alias_string.clone(), address);
        account.aliases.push(alias_string);
        Ok(())
    }

    /// Gets an account's data and address by an identifier (address or alias).
    pub fn get_account(&self, identifier: &str) -> Option<(&Account, Address)> {
        if let Ok(address) = identifier.parse::<Address>() {
            return self.accounts.get(&address).map(|acc| (acc, address));
        }

        self.aliases
            .get(identifier)
            .and_then(|&addr| self.accounts.get(&addr).map(|acc| (acc, addr)))
    }

    /// Lists all accounts in the wallet.
    pub fn list_accounts(&self) -> Vec<(Address, &Account)> {
        self.accounts
            .iter()
            .map(|(&addr, acc)| (addr, acc))
            .collect()
    }

    /// Sets the nonce for a specific account.
    pub fn set_nonce(&mut self, identifier: &str, nonce: u64) -> Result<()> {
        let (_, address) = self
            .get_account(identifier)
            .ok_or_else(|| WalletError::SignerNotFound(identifier.to_string()))?;

        if let Some(account) = self.accounts.get_mut(&address) {
            account.nonce = nonce;
            self.mark_dirty();
            Ok(())
        } else {
            Err(WalletError::AccountNotFound(address))
        }
    }

    /// Signs a transaction request with the specified account.
    ///
    /// This method also increments the nonce of the signing account upon success.
    pub async fn sign_transaction(
        &mut self,
        tx_request: &Eip1559TransactionRequest,
        from_identifier: &str,
    ) -> Result<SignedTransaction> {
        let (account, from_address) = self
            .get_account(from_identifier)
            .ok_or_else(|| WalletError::SignerNotFound(from_identifier.to_string()))?;

        // Validate the transaction nonce
        if tx_request.nonce != U256::from(account.nonce) {
            return Err(WalletError::NonceMismatch {
                expected: account.nonce,
                actual: tx_request.nonce.as_u64(),
            });
        }

        let signer = self.get_signer(&from_address)?;

        let typed_tx: TypedTransaction = tx_request.clone().into();
        let signature = signer.sign_transaction(&typed_tx).await?;

        // Increment the nonce after successful signing
        if let Some(account) = self.accounts.get_mut(&from_address) {
            account.nonce += 1;
            self.mark_dirty();
        } else {
            // This should ideally not happen if get_account succeeded
            return Err(WalletError::AccountNotFound(from_address));
        }

        let rlp_signed = typed_tx.rlp_signed(&signature);
        let hash = typed_tx.hash(&signature);

        Ok(SignedTransaction {
            raw_transaction: rlp_signed.to_vec(),
            hash: hash.into(),
            signature: (signature.v, signature.r.into(), signature.s.into()),
            chain_id: tx_request.chain_id,
        })
    }

    /// Gets the file path of the wallet.
    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    /// Sets the file path of the wallet.
    pub fn set_file_path<P: AsRef<Path>>(&mut self, path: P) {
        self.file_path = Some(path.as_ref().to_path_buf());
    }

    /// Checks if the wallet has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marks the wallet as having unsaved changes.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

/// Checks if an alias is valid (1-20 alphanumeric characters).
fn is_valid_alias(alias: &str) -> bool {
    !alias.is_empty()
        && alias.len() <= 20
        && alias.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}
