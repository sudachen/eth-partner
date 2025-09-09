use anyhow::Result;
use serde_json::{json, Map, Value};

// Pull in the test util to spawn anvil
#[path = "test_utils/anvil.rs"]
mod anvil;

use anvil::AnvilHandle;
use repl::config::{Config, WalletServerConfig};
use repl::tools::mcp_wallet::start_mcp_wallet_server;
use rmcp::model::CallToolRequestParam;

#[tokio::test]
async fn e2e_alias_unknown_address_creates_watch_only() -> Result<()> {
    // Start anvil and configure wallet server with empty wallet
    let handle = AnvilHandle::spawn_and_wait().await?;
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

    // Empty wallet file
    std::fs::write(
        &wallet_file,
        serde_json::to_string_pretty(&json!({
            "accounts": {},
            "aliases": {}
        }))?,
    )?;

    // Start wallet server and client
    let server = start_mcp_wallet_server(&cfg).await?;
    let (client_stream, shutdown) = server.into_client_stream();
    let client = rmcp::serve_client((), client_stream).await?;

    // Use an Anvil address that is not preloaded in wallet file
    let unknown_addr = "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"; // Anvil[2]
    let alias = "wo_alias_test";

    // Call set_alias
    let mut args = Map::new();
    args.insert("address".to_string(), json!(unknown_addr));
    args.insert("alias".to_string(), json!(alias));
    client
        .call_tool(CallToolRequestParam {
            name: "set_alias".into(),
            arguments: Some(args),
        })
        .await?;

    // Verify watch-only via list_accounts (is_signing = false)
    let list = client
        .call_tool(CallToolRequestParam {
            name: "list_accounts".into(),
            arguments: None,
        })
        .await?;
    let accounts: Vec<Value> = serde_json::from_value(list.structured_content.unwrap())?;
    let empty: Vec<Value> = Vec::new();
    let found = accounts.iter().any(|a| {
        let aliases = a["aliases"].as_array().unwrap_or(&empty);
        let has_alias = aliases.iter().any(|v| v.as_str() == Some(alias));
        let is_signing = a["is_signing"].as_bool().unwrap_or(true);
        has_alias && !is_signing
    });
    assert!(found, "expected watch-only account with alias present");

    // Shutdown
    client.cancel().await.ok();
    shutdown.shutdown();
    handle.stop().await.ok();
    Ok(())
}
