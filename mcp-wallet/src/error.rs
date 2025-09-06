use ethers::core::types::SignatureError;
use ethers::types::Address;
use ethers::utils::rlp::DecoderError;
use thiserror::Error;

/// Custom error type for the MCP Wallet.
#[derive(Debug, Error)]
pub enum WalletError {
    /// Error related to wallet file operations.
    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),

    /// Error related to JSON serialization/deserialization.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// A catch-all for general wallet errors.
    #[error("Wallet error: {0}")]
    WalletError(String),

    /// Error when an account is not found by its address.
    #[error("Account not found for address: {0}")]
    AccountNotFound(Address),

    /// Error when a signer cannot be found for a given identifier (address or alias).
    #[error("Signer not found for identifier: {0}")]
    SignerNotFound(String),

    /// Error when trying to add an account that already exists.
    #[error("Account already exists for address: {0}")]
    AccountAlreadyExists(Address),

    /// Error when a provided private key is invalid.
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),

    /// Error for invalid alias format.
    #[error("Alias '{0}' is invalid. It must be 1-20 alphanumeric characters.")]
    InvalidAlias(String),

    /// Error when an alias already exists.
    #[error("Alias '{0}' already exists.")]
    AliasAlreadyExists(String),

    /// Error when the transaction nonce does not match the account's nonce.
    #[error("Nonce mismatch: expected {expected}, but got {actual}")]
    NonceMismatch {
        /// The expected nonce of the account.
        expected: u64,
        /// The actual nonce provided in the transaction.
        actual: u64,
    },

    /// Error from the ethers-rs signer module.
    #[error("Ethers signer error: {0}")]
    EthersSignerError(#[from] ethers::signers::WalletError),

    /// Error from hex decoding.
    #[error("Hex decoding error: {0}")]
    FromHexError(#[from] hex::FromHexError),

    /// Error from RLP decoding.
    #[error("RLP decoding error: {0}")]
    RlpDecoderError(#[from] DecoderError),

    /// Error from signature operations.
    #[error("Signature error: {0}")]
    SignatureError(#[from] SignatureError),
}

/// Result type for wallet operations.
pub type Result<T, E = WalletError> = std::result::Result<T, E>;
