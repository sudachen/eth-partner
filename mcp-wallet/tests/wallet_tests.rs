use ethers::core::types::{Address, U256};
use mcp_wallet::{error::WalletError, transaction::TransactionBuilder, wallet::Wallet};

fn create_test_wallet() -> Wallet {
    let mut wallet = Wallet::new();
    wallet
        .import_private_key(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
            "testaccount",
        )
        .unwrap();
    wallet
}

#[test]
fn test_create_account() {
    let mut wallet = Wallet::new();
    let address = wallet.create_account("testaccount").unwrap();
    let address_str = format!("0x{:x}", address);
    assert!(wallet.get_account(&address_str).is_some());
}

#[test]
fn test_add_alias() {
    let mut wallet = Wallet::new();
    let address = wallet.create_account("mainaccount").unwrap();

    // Add valid alias
    wallet
        .add_alias(address, "secondaryalias".to_string())
        .unwrap();
    assert_eq!(
        wallet.get_account("secondaryalias").map(|(_, addr)| addr),
        Some(address)
    );

    // Add duplicate alias should fail
    let result = wallet.add_alias(address, "secondaryalias".to_string());
    assert!(matches!(result, Err(WalletError::AliasAlreadyExists(_))));

    // Add invalid alias should fail
    let result = wallet.add_alias(address, "invalid alias!".to_string());
    assert!(matches!(result, Err(WalletError::InvalidAlias(_))));
}

#[test]
fn test_get_account() {
    let mut wallet = Wallet::new();
    let address = wallet.create_account("testaccount").unwrap();

    let address_str = format!("0x{:x}", address);

    // Get by address
    assert!(wallet.get_account(&address_str).is_some());

    // Get by alias
    assert!(wallet.get_account("testaccount").is_some());

    // Non-existent account
    assert!(wallet.get_account("nonexistent").is_none());
}

#[test]
fn test_list_accounts() {
    let mut wallet = Wallet::new();
    let address1 = wallet.create_account("account1").unwrap();
    let address2 = wallet.create_account("account2").unwrap();

    let accounts = wallet.list_accounts();
    assert_eq!(accounts.len(), 2);

    let addresses: Vec<_> = accounts.iter().map(|(addr, _)| *addr).collect();
    assert!(addresses.contains(&address1));
    assert!(addresses.contains(&address2));
}

#[test]
fn test_create_and_get_account() {
    let mut wallet = Wallet::new();
    let alias = "mainaccount";
    let address = wallet.create_account(alias).unwrap();

    let (account, found_address) = wallet.get_account(alias).unwrap();
    assert_eq!(address, found_address);
    assert_eq!(account.aliases, vec![alias]);

    let (account_by_addr, _) = wallet.get_account(&format!("0x{:x}", address)).unwrap();
    assert_eq!(account.private_key, account_by_addr.private_key);
}

#[test]
fn test_add_alias_workflow() {
    let mut wallet = create_test_wallet();
    let (address, _) = wallet.list_accounts()[0];

    // Add valid alias
    wallet.add_alias(address, "secondary".to_string()).unwrap();
    let (account_reloaded, _) = wallet.get_account("secondary").unwrap();
    assert!(account_reloaded.aliases.contains(&"secondary".to_string()));

    // Add duplicate alias should fail
    let result = wallet.add_alias(address, "secondary".to_string());
    assert!(matches!(result, Err(WalletError::AliasAlreadyExists(_))));

    // Add invalid alias should fail
    let result = wallet.add_alias(address, "invalid alias!".to_string());
    assert!(matches!(result, Err(WalletError::InvalidAlias(_))));
}

#[test]
fn test_alias_uniqueness() {
    let mut wallet = Wallet::new();
    wallet.create_account("uniquealias").unwrap();
    let err = wallet.create_account("uniquealias").unwrap_err();
    assert!(matches!(err, WalletError::AliasAlreadyExists(_)));
}

#[tokio::test]
async fn test_sign_transaction() {
    let mut wallet = create_test_wallet();
    let (account, address) = wallet.get_account("testaccount").unwrap();
    let initial_nonce = account.nonce;

    let tx_request = TransactionBuilder::new()
        .chain_id(1)
        .to(Address::random())
        .value(U256::from(100))
        .nonce(U256::from(initial_nonce)) // Use the account's nonce
        .gas(U256::from(21000))
        .max_fee_per_gas(U256::from(20_000_000_000u64))
        .max_priority_fee_per_gas(U256::from(1_500_000_000u64))
        .build();

    let signed_tx = wallet
        .sign_transaction(&tx_request, "testaccount")
        .await
        .unwrap();

    let recovered_address = signed_tx.recover().unwrap();
    assert_eq!(recovered_address, address);
    assert_eq!(signed_tx.chain_id, 1);

    // Check if the nonce was incremented
    let (account_after, _) = wallet.get_account("testaccount").unwrap();
    assert_eq!(account_after.nonce, initial_nonce + 1);
}

#[tokio::test]
async fn test_sign_transaction_with_nonce_mismatch() {
    let mut wallet = create_test_wallet();
    let (account, _) = wallet.get_account("testaccount").unwrap();
    let incorrect_nonce = account.nonce + 1;

    let tx_request = TransactionBuilder::new()
        .chain_id(1)
        .to(Address::random())
        .value(U256::from(100))
        .nonce(U256::from(incorrect_nonce))
        .build();

    let result = wallet.sign_transaction(&tx_request, "testaccount").await;

    assert!(matches!(result, Err(WalletError::NonceMismatch { .. })));
}

#[test]
fn test_set_nonce() {
    let mut wallet = create_test_wallet();
    let new_nonce = 10;

    wallet.set_nonce("testaccount", new_nonce).unwrap();

    let (account, _) = wallet.get_account("testaccount").unwrap();
    assert_eq!(account.nonce, new_nonce);
}

#[test]
fn test_save_and_load_wallet() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test-wallet.json");

    // 1. Create a wallet, add an account, and save it
    let mut wallet_to_save = Wallet::new();
    wallet_to_save.set_file_path(&file_path);
    let original_address = wallet_to_save.create_account("saved_account").unwrap();

    let contents = serde_json::to_string_pretty(&wallet_to_save).unwrap();
    std::fs::write(&file_path, contents).unwrap();

    // 2. Load the wallet from the file
    let loaded_contents = std::fs::read_to_string(&file_path).unwrap();
    let loaded_wallet: Wallet = serde_json::from_str(&loaded_contents).unwrap();

    // 3. Verify the loaded wallet has the correct data
    let (account, address) = loaded_wallet.get_account("saved_account").unwrap();
    assert_eq!(address, original_address);
    assert_eq!(account.aliases, vec!["saved_account"]);
    assert_eq!(account.nonce, 0);
}
