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
async fn e2e_import_private_key_creates_signing_account() -> Result<()> {
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

    // Ensure no accounts initially
    let list_before = client
        .call_tool(CallToolRequestParam {
            name: "list_accounts".into(),
            arguments: None,
        })
        .await?;
    let accounts_before: Vec<Value> =
        serde_json::from_value(list_before.structured_content.unwrap())?;
    assert!(accounts_before.is_empty());

    // Import known Anvil private key (index 0)
    let mut args = Map::new();
    args.insert(
        "private_key".to_string(),
        json!("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"),
    );
    client
        .call_tool(CallToolRequestParam {
            name: "import_private_key".into(),
            arguments: Some(args),
        })
        .await?;

    // Verify signing account exists
    let list_after = client
        .call_tool(CallToolRequestParam {
            name: "list_accounts".into(),
            arguments: None,
        })
        .await?;
    let accounts_after: Vec<Value> =
        serde_json::from_value(list_after.structured_content.unwrap())?;
    assert!(
        accounts_after
            .iter()
            .any(|a| a["is_signing"].as_bool().unwrap_or(false)),
        "expected a signing account after import",
    );

    // Shutdown
    client.cancel().await.ok();
    shutdown.shutdown();
    handle.stop().await.ok();
    Ok(())
}
