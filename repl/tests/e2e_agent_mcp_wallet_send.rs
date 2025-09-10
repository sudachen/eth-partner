use anyhow::Result;
use regex::Regex;

// Pull in the test util to spawn anvil
#[path = "test_utils/anvil.rs"]
mod anvil;

use anvil::AnvilHandle;
use repl::agent::ReplAgent;
use repl::config::GenerationConfig;
use repl::config::{Config, WalletServerConfig};
use repl::tools::mcp_wallet::start_mcp_wallet_server;
use rig::agent::AgentBuilder;
use rig::client::{CompletionClient, ProviderClient};
use rig::providers::gemini;
use rmcp::model::CallToolRequestParam;
use serde_json::{json, Map};
use std::time::Duration;
use tokio::time::sleep;

/// NOTE: This is an end-to-end test that exercises a real LLM agent (Gemini)
/// driving the MCP wallet tools using natural language. It requires:
/// - GEMINI_API_KEY present in the environment
/// - Foundry's `anvil` available on PATH
/// - Network access for the LLM provider
/// Therefore this test is marked #[ignore] and should be run manually:
///   cargo test -p repl -- --ignored --test-threads=1
#[tokio::test]
#[ignore]
async fn e2e_agent_mcp_wallet_send_flow() -> Result<()> {
    dotenvy::dotenv().ok();

    // Fail early if GEMINI_API_KEY is not present
    if std::env::var("GEMINI_API_KEY").is_err() {
        eprintln!("GEMINI_API_KEY not set; skipping test.");
        return Ok(());
    }

    // 1) Start anvil
    let handle = AnvilHandle::spawn_and_wait().await?;

    // 2) Prepare config with wallet server enabled and pointing to anvil
    let tmp = tempfile::tempdir()?;
    let wallet_file = tmp.path().join(".wallet.json");

    // Also set env vars to mimic .env usage
    std::env::set_var("ETH_RPC_URL", &handle.url);
    std::env::set_var("CHAIN_ID", format!("{}", handle.chain_id));
    std::env::set_var("WALLET_FILE", wallet_file.to_string_lossy().to_string());

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

    // 3) Start the in-process MCP wallet server and tool client
    let server = start_mcp_wallet_server(&cfg).await?;
    let (client_stream, _shutdown) = server.into_client_stream();
    let mcp_client = rmcp::serve_client((), client_stream).await?;

    // 4) Prepare a real Gemini agent builder with our system prompt
    let gclient = gemini::Client::from_env();
    let mut agent_builder: AgentBuilder<_> = gclient
        .agent("gemini-2.0-flash")
        .preamble(include_str!("../../system-prompt.md"));
    // Provide minimal generationConfig required by provider
    let generation_config = GenerationConfig {
        temperature: 0.2,
        top_k: 1,
        top_p: 1.0,
        max_output_tokens: 2048,
        stop_sequences: vec![],
    };
    agent_builder = agent_builder.additional_params(json!({
        "generationConfig": generation_config
    }));

    // 5) Scripted conversation with the real agent
    // Prompts provided by user
    let alice_addr = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"; // Anvil[0]
    let bob_addr = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"; // Anvil[1]
    let pk_alice = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    // Build tool only after any direct RMCP preconditions and then build agent

    // Ensure Bob alias exists: try to resolve via RMCP, and if not found, set it directly
    let mut rargs = Map::new();
    rargs.insert("alias".to_string(), json!("Bob"));
    let needs_set = mcp_client
        .call_tool(CallToolRequestParam {
            name: "resolve_alias".into(),
            arguments: Some(rargs),
        })
        .await
        .is_err();
    if needs_set {
        let mut sargs = Map::new();
        sargs.insert("address".to_string(), json!(bob_addr));
        sargs.insert("alias".to_string(), json!("Bob"));
        let _ = mcp_client
            .call_tool(CallToolRequestParam {
                name: "set_alias".into(),
                arguments: Some(sargs),
            })
            .await?;
    }

    // Register wallet tool and create the agent
    let mcp_tool = repl::tools::mcp_wallet::McpWalletTool::new(mcp_client);
    agent_builder = agent_builder.tool(mcp_tool);
    let mut agent = Some(ReplAgent::new(agent_builder));

    // Probe provider quota before proceeding with expensive flow
    match repl::handle_line("ping".to_string(), &mut agent).await {
        Ok(_) => {}
        Err(e) => {
            let s = e.to_string();
            if s.contains("RESOURCE_EXHAUSTED") || s.contains("quota") || s.contains("429") {
                eprintln!("Provider quota exhausted; skipping test. Error: {}", s);
                return Ok(());
            } else {
                return Err(e);
            }
        }
    }

    // Step A: Alice alias
    let out = repl::handle_line(format!("Alice is {}", alice_addr), &mut agent).await?;
    assert!(out.is_some(), "agent produced no output for Alice alias");

    // Step B: Bob alias
    let out = repl::handle_line(format!("Bob is {}", bob_addr), &mut agent).await?;
    assert!(out.is_some(), "agent produced no output for Bob alias");

    // Step C: import key
    let out = repl::handle_line(format!("import key {}", pk_alice), &mut agent).await?;
    assert!(out.is_some(), "agent produced no output for import key");

    // Step D: list accounts and expect both aliases
    let out = repl::handle_line("list accounts".to_string(), &mut agent).await?;
    let resp = out.unwrap_or_default();
    // Be tolerant to agent formatting; ensure both names appear in the response
    assert!(resp.to_lowercase().contains("alice"));
    assert!(resp.to_lowercase().contains("bob"));

    println!("{resp}");

    // Step E: send 1 ETH from Alice to Bob
    let out = repl::handle_line("send 1 ETH from Alice to Bob".to_string(), &mut agent).await?;
    let resp = out.unwrap_or_default();

    // Look for a 0x-prefixed 32-byte hash (66 chars with 0x)
    let re = Regex::new(r"0x[0-9a-fA-F]{64}").unwrap();
    assert!(
        re.is_match(&resp),
        "expected tx hash in response, got: {}",
        resp
    );

    // Optional: give the node a moment to mine and then the agent might
    // provide a receipt/status in a follow-up. Not strictly required here.
    sleep(Duration::from_millis(250)).await;

    // If we got here, the flow worked
    Ok(())
}
