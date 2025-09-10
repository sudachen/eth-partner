use anyhow::Result;
use serde_json::{json, Map, Value};

// Pull in the test util to spawn anvil
#[path = "test_utils/anvil.rs"]
mod anvil;

use anvil::AnvilHandle;
use repl::config::{Config, WalletServerConfig};
use repl::tools::mcp_wallet::start_mcp_wallet_server;
use rmcp::model::CallToolRequestParam;
use rmcp::service::RunningService;

// NOTE: Sub-tasks 5.2 and 5.3 add concrete scenarios using this setup.

async fn setup_wallet_server_with_anvil(
) -> Result<(RunningService<rmcp::service::RoleClient, ()>, AnvilHandle)> {
    // Start anvil and set env for ETH RPC
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

    // Start the in-process MCP wallet server
    let server = start_mcp_wallet_server(&cfg).await?;

    // Build RMCP client from the server's client stream
    let (client_stream, shutdown) = server.into_client_stream();
    let client = rmcp::serve_client((), client_stream).await?;

    // Ensure server is alive by listing tools
    let list = client.list_tools(None).await?;
    assert!(list.tools.iter().any(|t| t.name == "resolve_alias"));

    // Attach shutdown to drop at the end of test path
    let _ = shutdown; // currently unused in scaffolding

    Ok((client, handle))
}

#[tokio::test]
async fn e2e_alias_resolution_not_found() -> Result<()> {
    let (client, handle) = setup_wallet_server_with_anvil().await?;

    // No alias created; attempt to resolve unknown alias
    let mut args = Map::new();
    args.insert("alias".to_string(), json!("does_not_exist"));
    let res = client
        .call_tool(CallToolRequestParam {
            name: "resolve_alias".into(),
            arguments: Some(args),
        })
        .await;
    assert!(res.is_err(), "expected error when alias is not found");

    // Shutdown
    if let Ok(c) = res {
        let _ = c;
    } // silence unused in success path (shouldn't happen)
    client.cancel().await.ok();
    handle.stop().await.ok();

    Ok(())
}

#[tokio::test]
async fn e2e_alias_resolution_success() -> Result<()> {
    let (client, handle) = setup_wallet_server_with_anvil().await?;

    // Create a new account with alias "AliCe"
    let mut args = Map::new();
    args.insert("alias".to_string(), json!("AliCe"));
    let new_account = client
        .call_tool(CallToolRequestParam {
            name: "new_account".into(),
            arguments: Some(args),
        })
        .await?;
    let acc_json: Value = new_account.structured_content.unwrap_or(Value::Null);
    let address = acc_json["address"].as_str().unwrap().to_string();

    // Resolve with lower and upper case via resolve_alias
    let mut args = Map::new();
    args.insert("alias".to_string(), json!("alice"));
    let r1 = client
        .call_tool(CallToolRequestParam {
            name: "resolve_alias".into(),
            arguments: Some(args),
        })
        .await?;
    let r1_addr = r1.structured_content.unwrap()["address"]
        .as_str()
        .unwrap()
        .to_string();

    let mut args = Map::new();
    args.insert("alias".to_string(), json!("ALICE"));
    let r2 = client
        .call_tool(CallToolRequestParam {
            name: "resolve_alias".into(),
            arguments: Some(args),
        })
        .await?;
    let r2_addr = r2.structured_content.unwrap()["address"]
        .as_str()
        .unwrap()
        .to_string();

    assert_eq!(r1_addr, r2_addr);
    assert_eq!(r1_addr, address);

    // Shutdown
    client.cancel().await.ok();
    handle.stop().await.ok();

    Ok(())
}
