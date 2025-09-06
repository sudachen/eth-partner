//! Tests for transaction creation and signing.

use ethers::core::types::{Address, U256};
use mcp_wallet::{
    commands::handle_mcp_command,
    models::Network,
    prelude::*,
    transaction::TransactionBuilder,
};
use serde_json::json;

const TEST_PRIVATE_KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

/// Helper function to create a test wallet with a known private key.
fn create_test_wallet() -> Wallet {
    let mut wallet = Wallet::new();
    wallet
        .import_private_key(TEST_PRIVATE_KEY, "testaccount")
        .unwrap();
    wallet
}

#[tokio::test]
async fn test_create_and_sign_transaction() {
    let mut wallet = create_test_wallet();
    let (account, address) = wallet.get_account("testaccount").unwrap();
    let nonce = account.nonce;

    let to = Address::random();
    let value = U256::from(1_000_000_000_000_000_000u128); // 1 ETH

    let tx_request = TransactionBuilder::new()
        .chain_id(Network::Local.chain_id())
        .to(to)
        .value(value)
        .gas(21000)
        .max_fee_per_gas(20_000_000_000u64) // 20 Gwei
        .max_priority_fee_per_gas(1_500_000_000u64) // 1.5 Gwei
        .nonce(nonce)
        .build();

    let signed_tx = wallet
        .sign_transaction(&tx_request, "testaccount")
        .await
        .unwrap();

    assert_eq!(signed_tx.chain_id, Network::Local.chain_id());
    assert!(!signed_tx.raw_transaction.is_empty());

    // Verify the signature
    let recovered_address = signed_tx.recover().unwrap();
    assert_eq!(recovered_address, address);

    // Verify nonce was incremented
    let (account_after, _) = wallet.get_account("testaccount").unwrap();
    assert_eq!(account_after.nonce, nonce + 1);
}

#[tokio::test]
async fn test_transaction_with_data() {
    let mut wallet = create_test_wallet();
    let (account, address) = wallet.get_account("testaccount").unwrap();

    let to = Address::random();
    let data = hex::decode("a9059cbb0000000000000000000000000000000000000000000000000000000000000001").unwrap();

    let tx_request = TransactionBuilder::new()
        .chain_id(Network::Local.chain_id())
        .to(to)
        .data(data)
        .gas(50000) // Gas is higher for tx with data
        .max_fee_per_gas(20_000_000_000u64)
        .max_priority_fee_per_gas(1_500_000_000u64)
        .nonce(account.nonce)
        .build();

    let signed_tx = wallet
        .sign_transaction(&tx_request, "testaccount")
        .await
        .unwrap();

    assert_eq!(signed_tx.chain_id, Network::Local.chain_id());
    let recovered_address = signed_tx.recover().unwrap();
    assert_eq!(recovered_address, address);
}

#[tokio::test]
async fn test_create_tx_command() {
    let mut wallet = create_test_wallet();
    let (_, _address) = wallet.get_account("testaccount").unwrap();

    let to = Address::random();
    let value = "1000000000000000000"; // 1 ETH

    let command = json!({
        "command": "create-tx",
        "params": {
            "from": "testaccount",
            "to": format!("0x{:x}", to),
            "value": value,
            "chain_id": Network::Local.chain_id(),
            "gas": 21000,
            "max_fee_per_gas": "20000000000",
            "max_priority_fee_per_gas": "1500000000"
        }
    });

    let response = handle_mcp_command(&command.to_string(), &mut wallet).await;

    assert_eq!(response.status, "success");
    let tx_json = response.data.unwrap();
    let tx_request: Eip1559TransactionRequest = serde_json::from_value(tx_json).unwrap();

    assert_eq!(tx_request.chain_id, Network::Local.chain_id());
    assert_eq!(tx_request.to, Some(to));
    assert_eq!(tx_request.value, U256::from_dec_str(value).unwrap());
    assert_eq!(tx_request.gas, U256::from(21000));
}
