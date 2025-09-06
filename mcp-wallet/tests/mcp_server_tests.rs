//! Integration tests for the MCP Wallet Server.

use assert_cmd::prelude::*;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use tempfile::tempdir;

#[test]
fn test_full_workflow() {
    // Use a temporary directory to isolate the wallet file
    let temp_dir = tempdir().unwrap();
    let home_dir = temp_dir.path();

    // Set the HOME env var to our temp dir so the wallet is created there
    let mut cmd = Command::cargo_bin("mcp-wallet").unwrap();
    let mut child = cmd
        .env("HOME", home_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to spawn mcp-wallet process");

    let mut writer = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());

    // 1. Create a new account
    let new_account_cmd = json!({
        "command": "new-account",
        "params": { "alias": "testaccount" }
    });
    writeln!(writer, "{}", new_account_cmd).unwrap();
    writer.flush().unwrap();

    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let response: Value = serde_json::from_str(&line).unwrap();

    assert_eq!(response["status"], "success");
    let address = response["data"]["address"].as_str().unwrap().to_string();
    assert!(address.starts_with("0x"));

    // 2. List accounts to verify creation
    let list_accounts_cmd = json!({ "command": "list-accounts" });
    writeln!(writer, "{}", list_accounts_cmd).unwrap();
    writer.flush().unwrap();

    line.clear();
    reader.read_line(&mut line).unwrap();
    let response: Value = serde_json::from_str(&line).unwrap();

    assert_eq!(response["status"], "success");
    assert_eq!(response["data"][0]["address"], address);
    assert_eq!(response["data"][0]["aliases"][0], "testaccount");
    assert_eq!(response["data"][0]["nonce"], 0);

    // 3. Create a transaction
    let create_tx_cmd = json!({
        "command": "create-tx",
        "params": {
            "from": "testaccount",
            "to": "0x0000000000000000000000000000000000000000",
            "value": "1000",
            "chain_id": 1,
            "gas": 21000,
            "max_fee_per_gas": "20000000000",
            "max_priority_fee_per_gas": "1500000000"
        }
    });
    writeln!(writer, "{}", create_tx_cmd).unwrap();
    writer.flush().unwrap();

    line.clear();
    reader.read_line(&mut line).unwrap();
    let response: Value = serde_json::from_str(&line).unwrap();

    assert_eq!(response["status"], "success");
    let tx_json = response["data"].clone();
    assert_eq!(tx_json["nonce"], "0x0");

    // 4. Sign the transaction
    let sign_tx_cmd = json!({
        "command": "sign-tx",
        "params": {
            "tx_json": tx_json,
            "from": "testaccount"
        }
    });
    writeln!(writer, "{}", sign_tx_cmd).unwrap();
    writer.flush().unwrap();

    line.clear();
    reader.read_line(&mut line).unwrap();
    let response: Value = serde_json::from_str(&line).unwrap();

    assert_eq!(response["status"], "success");
    assert!(response["data"]["raw_transaction"].is_string());
    assert!(response["data"]["hash"].is_string());

    // 5. List accounts again to check nonce update
    writeln!(writer, "{}", list_accounts_cmd).unwrap();
    writer.flush().unwrap();

    line.clear();
    reader.read_line(&mut line).unwrap();
    let response: Value = serde_json::from_str(&line).unwrap();

    assert_eq!(response["status"], "success");
    assert_eq!(response["data"][0]["nonce"], 1);

    child.kill().unwrap();
}
