//! Integration tests for the EthClient and related MCP tools.

use mcp_wallet::{eth_client::EthClient, service::WalletHandler, wallet::Wallet};
use rmcp::{model::CallToolRequestParam, serve_client, service::ServiceExt};
use serde_json::{json, Map};
use std::process::{Child, Command};
use std::sync::Arc;
use std::time::Duration;
use tokio::{io::duplex, sync::Mutex};

/// A guard for a running Anvil instance that ensures the process is killed on drop.
struct AnvilInstance {
    child: Child,
    rpc_url: String,
}

impl AnvilInstance {
    fn new() -> Self {
        let port = port_check::free_local_port().expect("Failed to find a free local port");
        let rpc_url = format!("http://127.0.0.1:{}", port);

        let child = Command::new("anvil")
            .arg("--port")
            .arg(port.to_string())
            .spawn()
            .expect("Failed to start anvil. Is it installed and in your PATH?");

        // Give Anvil a moment to start up
        std::thread::sleep(Duration::from_secs(1));

        Self { child, rpc_url }
    }

    fn rpc_url(&self) -> &str {
        &self.rpc_url
    }
}

impl Drop for AnvilInstance {
    fn drop(&mut self) {
        if let Err(e) = self.child.kill() {
            eprintln!("Failed to kill anvil process: {}", e);
        }
    }
}

#[tokio::test]
async fn test_eth_get_current_block() {
    // 1. Start Anvil
    let anvil = AnvilInstance::new();
    let rpc_url = anvil.rpc_url().to_string();

    // 2. Setup MCP Server and Client
    let (client_stream, server_stream) = duplex(1024);
    let wallet = Arc::new(Mutex::new(Wallet::new()));
    let eth_client = Arc::new(EthClient::new(&rpc_url).unwrap());

    let server_wallet = wallet.clone();
    tokio::spawn(async move {
        let server = WalletHandler::new(server_wallet, eth_client)
            .serve(server_stream)
            .await
            .unwrap();
        server.waiting().await.unwrap();
    });
    let client = serve_client((), client_stream).await.unwrap();

    // 3. Call the tool
    let result = client
        .call_tool(CallToolRequestParam {
            name: "eth_get_current_block".into(),
            arguments: None,
        })
        .await
        .expect("Failed to call eth_get_current_block");

    let result_value = result.structured_content.unwrap();
    assert_eq!(result_value["block_number"], 0);
}

#[tokio::test]
async fn test_eth_get_balance() {
    // 1. Start Anvil
    let anvil = AnvilInstance::new();
    let rpc_url = anvil.rpc_url().to_string();

    // 2. Setup MCP Server and Client
    let (client_stream, server_stream) = duplex(1024);
    let wallet = Arc::new(Mutex::new(Wallet::new()));
    let eth_client = Arc::new(EthClient::new(&rpc_url).unwrap());

    let server_wallet = wallet.clone();
    tokio::spawn(async move {
        let server = WalletHandler::new(server_wallet, eth_client)
            .serve(server_stream)
            .await
            .unwrap();
        server.waiting().await.unwrap();
    });
    let client = serve_client((), client_stream).await.unwrap();

    // 3. Call the tool
    let mut args = Map::new();
    // Anvil pre-funds some accounts. Let's check one.
    args.insert(
        "address".to_string(),
        json!("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"),
    );

    let result = client
        .call_tool(CallToolRequestParam {
            name: "eth_get_balance".into(),
            arguments: Some(args),
        })
        .await
        .expect("Failed to call eth_get_balance");

    let result_value = result.structured_content.unwrap();
    // Default Anvil balance is 10000 ETH, formatted with full precision
    assert_eq!(result_value["balance_eth"], "10000.000000000000000000");
}
