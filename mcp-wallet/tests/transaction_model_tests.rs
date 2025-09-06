use ethers::{
    core::rand::thread_rng,
    core::types::{transaction::eip2718::TypedTransaction, Address, U256},
    signers::{LocalWallet, Signer},
};
use mcp_wallet::models::Eip1559TransactionRequest;

#[test]
fn test_eip1559_transaction_creation() {
    let tx = Eip1559TransactionRequest::new(
        1,
        Some(Address::zero()),
        U256::from(1_000_000_000_000_000_000u128), // 1 ETH
        None,
    )
    .gas(21000)
    .max_fee_per_gas(U256::from(20_000_000_000u64)) // 20 gwei
    .max_priority_fee_per_gas(U256::from(1_500_000_000u64)); // 1.5 gwei

    assert_eq!(tx.chain_id, 1);
    assert_eq!(tx.to, Some(Address::zero()));
    assert_eq!(tx.value, U256::from(1_000_000_000_000_000_000u128));
    assert_eq!(tx.gas, U256::from(21000));
    assert_eq!(tx.max_fee_per_gas, U256::from(20_000_000_000u64));
    assert_eq!(tx.max_priority_fee_per_gas, U256::from(1_500_000_000u64));
}

#[tokio::test]
async fn test_transaction_signing() {
    let wallet = LocalWallet::new(&mut thread_rng());
    let address = wallet.address();

    let tx = Eip1559TransactionRequest::new(
        1,
        Some(Address::zero()),
        U256::from(1_000_000_000_000_000_000u128), // 1 ETH
        None,
    )
    .gas(21000)
    .nonce(0);

    let typed_tx: TypedTransaction = tx.into();
    let signature = wallet.sign_transaction(&typed_tx).await.unwrap();

    // Verify the signature
    let recovered = signature.recover(typed_tx.sighash()).unwrap();
    assert_eq!(recovered, address);
}
