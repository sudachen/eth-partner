use anyhow::Result;
use serde_json::{json, Map, Value};

// Pull in the test util to spawn anvil
#[path = "test_utils/anvil.rs"]
mod anvil;

use anvil::AnvilHandle;
use repl::config::{Config, WalletServerConfig};
use repl::tools::mcp_wallet::start_mcp_wallet_server;
use rmcp::model::CallToolRequestParam;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn e2e_alias_send_flow() -> Result<()> {
    // Start anvil
    let handle = AnvilHandle::spawn_and_wait().await?;

    // Prepare config with wallet server enabled and pointing to anvil
    let tmp = tempfile::tempdir()?;
    let wallet_file = tmp.path().join(".wallet.json");

    let cfg = Config {
        wallet_server: WalletServerConfig {
            enable: true,
            rpc_url: handle.url.clone(),
            chain_id: Some(handle.chain_id),
            wallet_file: Some(wallet_file.clone()),
            gas_limit: None,
            gas_price: None,
            listen_address: "127.0.0.1:0".to_string(),
        },
        ..Default::default()
    };

    // Start wallet server
    let server = start_mcp_wallet_server(&cfg).await?;
    let (client_stream, shutdown) = server.into_client_stream();
    let client = rmcp::serve_client((), client_stream).await?;

    // --- Scripted user flow ---
    // >> Alice is 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 (Anvil[0])
    // >> Bob is 0x70997970C51812dc3A010C7d01b50e0d17dc79C8 (Anvil[1])
    let alice_addr = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
    let bob_addr = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";

    // Set aliases
    let mut aargs = Map::new();
    aargs.insert("address".to_string(), json!(alice_addr));
    aargs.insert("alias".to_string(), json!("Alice"));
    client
        .call_tool(CallToolRequestParam {
            name: "set_alias".into(),
            arguments: Some(aargs),
        })
        .await?;

    let mut bargs = Map::new();
    bargs.insert("address".to_string(), json!(bob_addr));
    bargs.insert("alias".to_string(), json!("Bob"));
    client
        .call_tool(CallToolRequestParam {
            name: "set_alias".into(),
            arguments: Some(bargs),
        })
        .await?;

    // >> import key 0x... (Anvil[0] private key) to upgrade Alice to signing
    let pk_alice = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let mut pkargs = Map::new();
    pkargs.insert("private_key".to_string(), json!(pk_alice));
    client
        .call_tool(CallToolRequestParam {
            name: "import_private_key".into(),
            arguments: Some(pkargs),
        })
        .await?;

    // >> list accounts (must include both Alice and Bob)
    let listed = client
        .call_tool(CallToolRequestParam {
            name: "list_accounts".into(),
            arguments: None,
        })
        .await?;
    let accounts: Vec<Value> = serde_json::from_value(listed.structured_content.unwrap())?;

    let mut has_alice = false;
    let mut has_bob = false;
    for a in &accounts {
        let aliases = a["aliases"].as_array().cloned().unwrap_or_default();
        if aliases.iter().any(|v| v.as_str() == Some("Alice")) {
            has_alice = true;
        }
        if aliases.iter().any(|v| v.as_str() == Some("Bob")) {
            has_bob = true;
        }
    }
    assert!(
        has_alice && has_bob,
        "expected both Alice and Bob to be listed"
    );

    // >> send 1 ETH from Alice to Bob
    let mut targs = Map::new();
    targs.insert("from".to_string(), json!("Alice"));
    targs.insert("to".to_string(), json!("Bob"));
    targs.insert("value_eth".to_string(), json!(1.0));
    targs.insert("chain_id".to_string(), json!(handle.chain_id));

    let tx_res = client
        .call_tool(CallToolRequestParam {
            name: "eth_transfer_eth".into(),
            arguments: Some(targs),
        })
        .await?;
    let tx_json = tx_res.structured_content.unwrap_or(Value::Null);
    let tx_hash = tx_json["transaction_hash"].as_str().unwrap_or("");
    assert!(
        tx_hash.starts_with("0x") && tx_hash.len() >= 66,
        "invalid tx hash: {}",
        tx_hash
    );

    // Wait for receipt and assert success
    let mut attempts = 0u32;
    let receipt_status = loop {
        let mut rargs = Map::new();
        rargs.insert("transaction_hash".to_string(), json!(tx_hash));
        let rcpt = client
            .call_tool(CallToolRequestParam {
                name: "eth_get_transaction_receipt".into(),
                arguments: Some(rargs),
            })
            .await?;
        let rcpt_json: Value = rcpt.structured_content.unwrap_or(Value::Null);
        if rcpt_json["found"].as_bool().unwrap_or(false) {
            break rcpt_json["status"].as_str().unwrap_or("").to_string();
        }
        if attempts > 20 {
            panic!("timed out waiting for transaction receipt");
        }
        attempts += 1;
        sleep(Duration::from_millis(250)).await;
    };
    assert_eq!(receipt_status, "success", "expected success receipt status");

    // Shutdown
    client.cancel().await.ok();
    shutdown.shutdown();
    handle.stop().await.ok();

    Ok(())
}
