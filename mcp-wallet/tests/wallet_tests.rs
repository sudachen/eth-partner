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