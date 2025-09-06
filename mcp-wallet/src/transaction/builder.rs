//! Transaction builder for creating EIP-1559 transactions

use crate::models::transaction::Eip1559TransactionRequest;
use ethers::core::types::{Address, U256};

/// Builder for creating EIP-1559 transactions.
///
/// This builder helps construct an `Eip1559TransactionRequest` by providing a
/// fluent interface for setting transaction parameters. Once built, the request
/// can be signed by a wallet.
#[derive(Debug, Clone, Default)]
pub struct TransactionBuilder {
    chain_id: Option<u64>,
    to: Option<Address>,
    value: Option<U256>,
    data: Option<Vec<u8>>,
    gas: Option<U256>,
    max_fee_per_gas: Option<U256>,
    max_priority_fee_per_gas: Option<U256>,
    nonce: Option<U256>,
}

impl TransactionBuilder {
    /// Creates a new transaction builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the chain ID for the transaction.
    pub fn chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Sets the recipient address.
    pub fn to(mut self, to: Address) -> Self {
        self.to = Some(to);
        self
    }

    /// Sets the amount to send in wei.
    pub fn value(mut self, value: impl Into<U256>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Sets the transaction data.
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// Sets the gas limit.
    pub fn gas(mut self, gas: impl Into<U256>) -> Self {
        self.gas = Some(gas.into());
        self
    }

    /// Sets the max fee per gas (in wei).
    pub fn max_fee_per_gas(mut self, max_fee: impl Into<U256>) -> Self {
        self.max_fee_per_gas = Some(max_fee.into());
        self
    }

    /// Sets the max priority fee per gas (in wei).
    pub fn max_priority_fee_per_gas(mut self, max_priority_fee: impl Into<U256>) -> Self {
        self.max_priority_fee_per_gas = Some(max_priority_fee.into());
        self
    }

    /// Sets the transaction nonce.
    pub fn nonce(mut self, nonce: impl Into<U256>) -> Self {
        self.nonce = Some(nonce.into());
        self
    }

    /// Builds the EIP-1559 transaction request.
    ///
    /// # Panics
    ///
    /// Panics if any of the required fields are not set.
    pub fn build(self) -> Eip1559TransactionRequest {
        Eip1559TransactionRequest {
            chain_id: self.chain_id.expect("chain_id is required"),
            to: self.to,
            value: self.value.unwrap_or_default(),
            data: self.data,
            gas: self.gas.unwrap_or_else(|| U256::from(21000)), // Default gas for a simple transfer
            max_fee_per_gas: self.max_fee_per_gas.unwrap_or_default(),
            max_priority_fee_per_gas: self.max_priority_fee_per_gas.unwrap_or_default(),
            nonce: self.nonce.expect("nonce is required"),
            access_list: Vec::new(), // Access list is not supported yet
        }
    }
}
