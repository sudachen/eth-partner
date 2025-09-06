//! Transaction data structures and related functionality

use crate::error::WalletError;
use ethers::{
    core::types::{transaction::eip2718::TypedTransaction, U256},
    types::{Address, Eip1559TransactionRequest as EthersEip1559TransactionRequest},
    utils::rlp,
};
use serde::{Deserialize, Serialize};

/// Represents an EIP-1559 transaction request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Eip1559TransactionRequest {
    /// Chain ID for the transaction
    pub chain_id: u64,
    /// Recipient address (None for contract creation)
    pub to: Option<Address>,
    /// Amount of ETH to send in wei
    pub value: U256,
    /// Transaction data (for contract interactions)
    pub data: Option<Vec<u8>>,
    /// Gas limit for the transaction
    pub gas: U256,
    /// Max fee per gas (in wei)
    pub max_fee_per_gas: U256,
    /// Max priority fee per gas (in wei)
    pub max_priority_fee_per_gas: U256,
    /// Transaction nonce
    pub nonce: U256,
    /// Access list for the transaction (EIP-2930)
    pub access_list: Vec<(Address, Vec<[u8; 32]>)>, // Simplified for now
}

impl Default for Eip1559TransactionRequest {
    fn default() -> Self {
        Self {
            chain_id: 1, // Default to mainnet
            to: None,
            value: U256::zero(),
            data: None,
            gas: U256::from(21000), // Default gas limit for simple transfer
            max_fee_per_gas: U256::from(20_000_000_000u64), // 20 gwei
            max_priority_fee_per_gas: U256::from(1_500_000_000u64), // 1.5 gwei
            nonce: U256::zero(),
            access_list: Vec::new(),
        }
    }
}

impl Eip1559TransactionRequest {
    /// Creates a new EIP-1559 transaction request
    pub fn new(
        chain_id: u64,
        to: Option<Address>,
        value: impl Into<U256>,
        data: Option<Vec<u8>>,
    ) -> Self {
        Self {
            chain_id,
            to,
            value: value.into(),
            data,
            ..Default::default()
        }
    }

    /// Sets the gas limit for the transaction
    pub fn gas(mut self, gas: impl Into<U256>) -> Self {
        self.gas = gas.into();
        self
    }

    /// Sets the max fee per gas
    pub fn max_fee_per_gas(mut self, max_fee: impl Into<U256>) -> Self {
        self.max_fee_per_gas = max_fee.into();
        self
    }

    /// Sets the max priority fee per gas
    pub fn max_priority_fee_per_gas(mut self, max_priority_fee: impl Into<U256>) -> Self {
        self.max_priority_fee_per_gas = max_priority_fee.into();
        self
    }

    /// Sets the transaction nonce
    pub fn nonce(mut self, nonce: impl Into<U256>) -> Self {
        self.nonce = nonce.into();
        self
    }
}

/// Converts the internal transaction request to the `ethers` equivalent.
impl From<Eip1559TransactionRequest> for TypedTransaction {
    fn from(tx: Eip1559TransactionRequest) -> Self {
        let tx = EthersEip1559TransactionRequest {
            to: tx.to.map(Into::into),
            value: Some(tx.value),
            data: tx.data.map(Into::into),
            nonce: Some(tx.nonce),
            gas: Some(tx.gas),
            max_fee_per_gas: Some(tx.max_fee_per_gas),
            max_priority_fee_per_gas: Some(tx.max_priority_fee_per_gas),
            chain_id: Some(tx.chain_id.into()),
            from: None,
            access_list: Default::default(), // Simplified for now
        };

        TypedTransaction::Eip1559(tx)
    }
}

/// Represents a signed transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignedTransaction {
    /// The raw transaction bytes
    pub raw_transaction: Vec<u8>,
    /// The transaction hash
    pub hash: [u8; 32],
    /// The signature components (v, r, s)
    pub signature: (u64, [u8; 32], [u8; 32]),
    /// The chain ID the transaction is valid for
    pub chain_id: u64,
}

impl SignedTransaction {
    /// Recovers the sender's address from the signature
    pub fn recover(&self) -> Result<Address, WalletError> {
        let tx: TypedTransaction = rlp::decode(&self.raw_transaction)?;
        let signature = ethers::core::types::Signature {
            r: self.signature.1.into(),
            s: self.signature.2.into(),
            v: self.signature.0,
        };

        let recovered_address = signature.recover(tx.sighash())?;
        Ok(recovered_address)
    }
}
