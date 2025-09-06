use mcp_wallet::{
    error::WalletError,
    wallet::Wallet,
};

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
    wallet.add_alias(address, "secondaryalias".to_string()).unwrap();
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