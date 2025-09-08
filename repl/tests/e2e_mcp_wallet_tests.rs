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
async fn e2e_mcp_wallet_server_tools() -> Result<()> {
    // 1) Start anvil and set env for ETH RPC
    let handle = AnvilHandle::spawn_and_wait().await?;

    // 2) Prepare config with wallet server enabled and pointing to anvil
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

    // Pre-populate wallet file with a known rich Anvil account under alias "rich"
    // Anvil[0] address and private key (from Anvil banner)
    let rich_addr = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
    let rich_pk = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"; // no 0x
    let mut accounts_map: Map<String, Value> = Map::new();
    accounts_map.insert(
        rich_addr.to_string(),
        json!({
            "private_key": rich_pk,
            "nonce": 0,
            "aliases": ["rich"]
        }),
    );
    let wallet_json = json!({
        "accounts": accounts_map,
        "aliases": { "rich": rich_addr }
    });

    std::fs::write(&wallet_file, serde_json::to_string_pretty(&wallet_json)?)?;

    // 3) Start the in-process MCP wallet server
    let server = start_mcp_wallet_server(&cfg).await?;

    // 4) Build RMCP client from the server's client stream
    let (client_stream, shutdown) = server.into_client_stream();
    let client = rmcp::serve_client((), client_stream).await?;

    // 5) List tools to verify server is alive
    let list = client.list_tools(None).await?;
    //println!("{list:?}");
    assert!(list.tools.iter().any(|t| t.name == "new_account"));
    assert!(list.tools.iter().any(|t| t.name == "eth_get_balance"));

    // 6) Call new_account to create a signer (alias: testaccount)
    let mut args = Map::new();
    args.insert("alias".to_string(), json!("testaccount"));
    let new_account = client
        .call_tool(CallToolRequestParam {
            name: "new_account".into(),
            arguments: Some(args),
        })
        .await?;
    let new_acc_json: Value = new_account.structured_content.unwrap_or(Value::Null);
    let address = new_acc_json["address"].as_str().unwrap().to_string();

    // 7) Query balance of a known anvil funded account via eth_get_balance
    let mut args = Map::new();
    args.insert(
        "address".to_string(),
        json!("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"),
    );
    let bal = client
        .call_tool(CallToolRequestParam {
            name: "eth_get_balance".into(),
            arguments: Some(args),
        })
        .await?;
    //println!("{bal:?}");
    let val: Value = bal.structured_content.unwrap_or(Value::Null);
    assert!(
        val["balance_eth"].is_string(),
        "expected string balance_eth, got: {:?}",
        val
    );

    // 7.5) Transfer 0.1 ETH from rich -> new_account
    let to_address = address.clone();
    let mut targs = Map::new();
    targs.insert("from".to_string(), json!("rich"));
    targs.insert("to".to_string(), json!(to_address));
    targs.insert("value_eth".to_string(), json!(0.1));
    targs.insert("chain_id".to_string(), json!(handle.chain_id));

    let tx_res = client
        .call_tool(CallToolRequestParam {
            name: "eth_transfer_eth".into(),
            arguments: Some(targs),
        })
        .await?;
    let tx_json = tx_res.structured_content.unwrap_or(Value::Null);
    assert!(tx_json["transaction_hash"].is_string());
    let tx_hash = tx_json["transaction_hash"].as_str().unwrap().to_string();

    // 7.55) Wait for receipt and assert success
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

    // 7.6) Check new account balance increased
    let mut args = Map::new();
    args.insert("address".to_string(), json!(address));
    let bal2 = client
        .call_tool(CallToolRequestParam {
            name: "eth_get_balance".into(),
            arguments: Some(args),
        })
        .await?;
    let val2: Value = bal2.structured_content.unwrap_or(Value::Null);
    let bal_str = val2["balance_eth"].as_str().unwrap_or("");
    // Should be > 0
    assert!(
        bal_str.parse::<f64>().unwrap_or(0.0) > 0.0,
        "recipient balance should increase"
    );

    // 8) Shutdown
    client.cancel().await.ok();
    shutdown.shutdown();
    handle.stop().await.ok();

    Ok(())
}
