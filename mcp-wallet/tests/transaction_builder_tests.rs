use mcp_wallet::transaction::TransactionBuilder;
use ethers::core::types::{Address, U256};

#[test]
fn test_transaction_builder_builds_correctly() {
    let to_address = Address::random();
    let value = U256::from(1000);
    let gas = U256::from(21000);
    let max_fee_per_gas = U256::from(20_000_000_000u64);
    let max_priority_fee_per_gas = U256::from(1_500_000_000u64);
    let nonce = U256::from(1);

    let tx_request = TransactionBuilder::new()
        .chain_id(1)
        .to(to_address)
        .value(value)
        .gas(gas)
        .max_fee_per_gas(max_fee_per_gas)
        .max_priority_fee_per_gas(max_priority_fee_per_gas)
        .nonce(nonce)
        .build();

    assert_eq!(tx_request.chain_id, 1);
    assert_eq!(tx_request.to, Some(to_address));
    assert_eq!(tx_request.value, value);
    assert_eq!(tx_request.gas, gas);
    assert_eq!(tx_request.max_fee_per_gas, max_fee_per_gas);
    assert_eq!(
        tx_request.max_priority_fee_per_gas,
        max_priority_fee_per_gas
    );
    assert_eq!(tx_request.nonce, nonce);
    assert!(tx_request.data.is_none());
}

#[test]
fn test_transaction_builder_defaults_gas_fields() {
    let tx_request = TransactionBuilder::new()
        .chain_id(1)
        .nonce(0)
        .build();

    assert_eq!(tx_request.gas, U256::from(21000));
    assert_eq!(tx_request.max_fee_per_gas, U256::zero());
    assert_eq!(tx_request.max_priority_fee_per_gas, U256::zero());
}

#[test]
#[should_panic(expected = "chain_id is required")]
fn test_transaction_builder_panics_if_chain_id_is_missing() {
    TransactionBuilder::new().nonce(1).build();
}

#[test]
#[should_panic(expected = "nonce is required")]
fn test_transaction_builder_panics_if_nonce_is_missing() {
    TransactionBuilder::new().chain_id(1).build();
}
