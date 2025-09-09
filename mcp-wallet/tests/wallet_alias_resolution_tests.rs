//! Tests for case-insensitive alias resolution helper in Wallet

use std::str::FromStr;

use ethers::types::Address;
use mcp_wallet::wallet::Wallet;

#[test]
fn resolve_alias_case_insensitive_happy_path() {
    let mut wallet = Wallet::new();

    let addr = Address::from_str("0x000000000000000000000000000000000000dEaD").unwrap();

    // Add alias with mixed case
    wallet
        .add_alias(addr, "AliCe".to_string())
        .expect("add_alias");

    // Resolve with different cases
    let r1 = wallet
        .resolve_alias_case_insensitive("alice")
        .expect("should resolve");
    let r2 = wallet
        .resolve_alias_case_insensitive("ALICE")
        .expect("should resolve");
    let r3 = wallet
        .resolve_alias_case_insensitive("AlIcE")
        .expect("should resolve");

    assert_eq!(r1, addr);
    assert_eq!(r2, addr);
    assert_eq!(r3, addr);
}

#[test]
fn resolve_alias_case_insensitive_not_found() {
    let wallet = Wallet::new();

    let none = wallet.resolve_alias_case_insensitive("unknown_alias");
    assert!(none.is_none());
}
