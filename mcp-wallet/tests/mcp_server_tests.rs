//! Integration tests for the fully compliant MCP Wallet Server.

use mcp_wallet::{eth_client::EthClient, service::WalletHandler, wallet::Wallet};
use rmcp::{model::CallToolRequestParam, serve_client, service::ServiceExt};
use serde_json::{json, Map, Value};
use std::sync::Arc;
use tokio::{io::duplex, sync::Mutex};

#[tokio::test]
async fn test_mcp_client_workflow() {
    // 1. Setup: Create an in-memory transport and start the server in a background task.
    let (client_stream, server_stream) = duplex(1024);

    // Create a new wallet and handler
    let wallet = Arc::new(Mutex::new(Wallet::new()));
    let eth_client = Arc::new(EthClient::new("http://127.0.0.1:8545").unwrap());

    // Spawn the server to run in the background
    let server_wallet = wallet.clone();
    let server_eth_client = eth_client.clone();
    tokio::spawn(async move {
        let server = WalletHandler::new(server_wallet, server_eth_client)
            .serve(server_stream)
            .await
            .unwrap();
        server.waiting().await.unwrap();
        eprintln!("mcp server finished")
    });

    // Create an MCP client connected to the in-memory stream
    // <DON'T CHANGE THIS LINE> it how client is creating
    let client = serve_client((), client_stream).await.unwrap();

    let nfo = client.peer_info();
    println!("{:?}", nfo);

    // 2. List tools to verify server is running and self-describing
    let list_tools_result = client.list_tools(None).await.expect("Failed to list tools");
    assert!(list_tools_result.tools.len() >= 5);
    let new_account_tool = list_tools_result
        .tools
        .iter()
        .find(|t| t.name == "new_account")
        .expect("new-account tool not found");
    assert_eq!(new_account_tool.name, "new_account");

    // 3. Create a new account
    let mut args = Map::new();
    args.insert("alias".to_string(), json!("testaccount"));
    let new_account_result = client
        .call_tool(CallToolRequestParam {
            name: "new_account".into(),
            arguments: Some(args),
        })
        .await
        .expect("Failed to call new-account");

    let result_value = new_account_result.structured_content.unwrap();
    let address = result_value["address"].as_str().unwrap().to_string();
    assert!(address.starts_with("0x"));

    // 4. List accounts to verify creation
    let list_accounts_result = client
        .call_tool(CallToolRequestParam {
            name: "list_accounts".into(),
            arguments: None,
        })
        .await
        .expect("Failed to call list-accounts");

    let accounts_value = list_accounts_result.structured_content.unwrap();
    let accounts: Vec<Value> = serde_json::from_value(accounts_value).unwrap();
    assert_eq!(accounts[0]["address"], address);
    assert_eq!(accounts[0]["aliases"][0], "testaccount");
    assert_eq!(accounts[0]["nonce"], 0);

    // 5. Create a transaction
    let mut args = Map::new();
    args.insert("from".to_string(), json!("testaccount"));
    args.insert(
        "to".to_string(),
        json!("0x0000000000000000000000000000000000000000"),
    );
    args.insert("value".to_string(), json!("1000"));
    args.insert("chain_id".to_string(), json!(1));
    let create_tx_result = client
        .call_tool(CallToolRequestParam {
            name: "create_tx".into(),
            arguments: Some(args),
        })
        .await
        .expect("Failed to call create-tx");

    let tx_json = create_tx_result.structured_content.unwrap();
    assert_eq!(tx_json["nonce"], "0x0");

    // 6. Sign the transaction
    let mut args = Map::new();
    args.insert("from".to_string(), json!("testaccount"));
    args.insert("tx_json".to_string(), tx_json);
    let sign_tx_result = client
        .call_tool(CallToolRequestParam {
            name: "sign_tx".into(),
            arguments: Some(args),
        })
        .await
        .expect("Failed to call sign-tx");

    let signed_tx = sign_tx_result.structured_content.unwrap();
    assert!(signed_tx["raw_transaction"].is_string());
    assert!(signed_tx["hash"].is_string());

    // 7. List accounts again to check nonce update
    let list_accounts_result_after = client
        .call_tool(CallToolRequestParam {
            name: "list_accounts".into(),
            arguments: None,
        })
        .await
        .expect("Failed to call list-accounts again");

    let accounts_json_after = list_accounts_result_after.structured_content.unwrap();
    let accounts_after: Vec<Value> = serde_json::from_value(accounts_json_after).unwrap();
    assert_eq!(accounts_after[0]["nonce"], 1);

    // 8. Shutdown
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_import_private_key_validation_errors() {
    // Setup server and client over in-memory transport
    let (client_stream, server_stream) = duplex(1024);
    let wallet = Arc::new(Mutex::new(Wallet::new()));
    let eth_client = Arc::new(EthClient::new("http://127.0.0.1:8545").unwrap());

    let server_wallet = wallet.clone();
    let server_eth_client = eth_client.clone();
    tokio::spawn(async move {
        let server = WalletHandler::new(server_wallet, server_eth_client)
            .serve(server_stream)
            .await
            .unwrap();
        server.waiting().await.unwrap();
    });

    let client = serve_client((), client_stream).await.unwrap();

    // Too short
    let mut args = Map::new();
    args.insert("private_key".to_string(), json!("0x1234"));
    let res = client
        .call_tool(rmcp::model::CallToolRequestParam {
            name: "import_private_key".into(),
            arguments: Some(args),
        })
        .await;
    assert!(res.is_err(), "expected error for short private key");

    // Non-hex
    let mut args = Map::new();
    args.insert(
        "private_key".to_string(),
        json!("0xZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ"),
    );
    let res = client
        .call_tool(rmcp::model::CallToolRequestParam {
            name: "import_private_key".into(),
            arguments: Some(args),
        })
        .await;
    assert!(res.is_err(), "expected error for non-hex private key");

    // All zeros
    let mut args = Map::new();
    args.insert(
        "private_key".to_string(),
        json!("0000000000000000000000000000000000000000000000000000000000000000"),
    );
    let res = client
        .call_tool(rmcp::model::CallToolRequestParam {
            name: "import_private_key".into(),
            arguments: Some(args),
        })
        .await;
    assert!(res.is_err(), "expected error for all-zero private key");

    client.cancel().await.unwrap();
}
