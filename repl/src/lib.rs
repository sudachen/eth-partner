//! The main library for the REPL AI assistant.

#[allow(dead_code)]
pub mod agent;
pub mod config;
pub mod tools;

use crate::agent::ReplAgent;
use crate::config::GenerationConfig;
use crate::tools::mcp_wallet::{start_mcp_wallet_server, McpWalletTool, ServerShutdown};
use crate::tools::web_search::WebSearchTool;
use anyhow::{Context, Result};
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::CompletionModel;
use rig::providers::gemini;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::Editor;
use serde_json::json;
use tracing::{info, warn};
use tracing_subscriber::{fmt, EnvFilter};

/// Runs the main REPL loop.
#[allow(dead_code)]
pub async fn run_repl() -> Result<()> {
    dotenvy::dotenv().ok();

    // Initialize logging (default to info if RUST_LOG not set)
    if tracing::subscriber::set_global_default(
        fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .with_target(false)
            .finish(),
    )
    .is_err()
    {
        // If already set by tests or another init, ignore.
    }

    /// Starts the embedded mcp-wallet server from the provided config and returns
    /// both a shutdown handle and an initialized RMCP client connected to it.
    ///
    /// This mirrors the REPL startup path so tests can verify REPL integration
    /// without entering the interactive loop.
    pub async fn start_mcp_wallet_and_client(
        config: &config::Config,
    ) -> Result<(
        crate::tools::mcp_wallet::ServerShutdown,
        rmcp::service::RunningService<rmcp::RoleClient, ()>,
    )> {
        use crate::tools::mcp_wallet::start_mcp_wallet_server;

        let handle = start_mcp_wallet_server(config).await?;
        let (client_stream, shutdown) = handle.into_client_stream();
        let client = rmcp::serve_client((), client_stream).await?;
        Ok((shutdown, client))
    }

    let config = config::load().context("Failed to load configuration")?;
    println!("Loaded config: {:?}", config);
    info!("Configuration loaded successfully");

    // --- Optional: Start embedded mcp-wallet MCP server ---
    let mut wallet_shutdown: Option<ServerShutdown> = None;
    let mut mcp_wallet_tool: Option<McpWalletTool> = None;
    if config.wallet_server.enable {
        match start_mcp_wallet_server(&config).await {
            Ok(handle) => {
                // Build RMCP client from the client stream and construct the pass-through tool
                let (client_stream, shutdown) = handle.into_client_stream();
                wallet_shutdown = Some(shutdown);
                match rmcp::serve_client((), client_stream).await {
                    Ok(client) => {
                        mcp_wallet_tool = Some(McpWalletTool::new(client));
                        info!("mcp-wallet server started (in-process) and client initialized");
                    }
                    Err(e) => {
                        return Err(e).context("Failed to initialize RMCP client for mcp-wallet");
                    }
                }
            }
            Err(e) => {
                // Per PRD: fail REPL startup on server failure (PoC behavior)
                return Err(e).context("Failed to start embedded mcp-wallet server");
            }
        }
    }

    // --- Agent Setup ---
    // Prioritize API key from environment, then fall back to config file.
    let api_key = std::env::var("GEMINI_API_KEY")
        .ok()
        .or(config.llm.google_api_key);

    let mut agent = if let Some(key) = api_key {
        std::env::set_var("GEMINI_API_KEY", key);
        let client = gemini::Client::from_env();
        println!("Gemini client initialized.");
        info!("Gemini client initialized");

        let mut agent_builder = client
            .agent("gemini-1.5-flash-latest")
            .preamble(include_str!("../../system-prompt.md"));

        let generation_config = config.llm.generation_config.unwrap_or_else(|| {
            println!("Using default generation config");
            GenerationConfig {
                temperature: 0.9,
                top_k: 1,
                top_p: 1.0,
                max_output_tokens: 2048,
                stop_sequences: vec![],
            }
        });

        agent_builder = agent_builder.additional_params(json!({
            "generationConfig": generation_config
        }));

        // Validate Google CSE configuration (without registering yet; see task 2.3)
        let google_search_api_key = std::env::var("GOOGLE_SEARCH_API_KEY")
            .ok()
            .or(config.tools.google_search_api_key);
        let google_search_engine_id = std::env::var("GOOGLE_SEARCH_ENGINE_ID")
            .ok()
            .or(config.tools.google_search_engine_id);

        match (google_search_api_key, google_search_engine_id) {
            (Some(api_key), Some(engine_id)) => {
                agent_builder = agent_builder.tool(WebSearchTool::new(api_key, engine_id));
                println!("Web search tool initialized (Google credentials detected).");
                info!("Web search tool enabled");
            }
            _ => {
                println!(
                    "Web search tool unavailable: missing GOOGLE_SEARCH_API_KEY and/or GOOGLE_SEARCH_ENGINE_ID."
                );
                warn!("Web search tool disabled; missing credentials");
            }
        }

        // Register MCP wallet tool if available
        if let Some(wallet_tool) = mcp_wallet_tool {
            agent_builder = agent_builder.tool(wallet_tool);
            info!("MCP wallet tool registered");
        } else if config.wallet_server.enable {
            // If enabled but tool not available, log a warning (shouldn't happen due to fail-fast above)
            warn!("MCP wallet tool not available despite enable=true");
        }

        Some(ReplAgent::new(agent_builder))
    } else {
        println!(
            "Warning: GEMINI_API_KEY not found in environment or config. LLM functionality will be disabled."
        );
        warn!("GEMINI_API_KEY not found; agent disabled");
        None
    };

    // --- REPL Loop ---
    let mut rl = Editor::<(), DefaultHistory>::new().context("Failed to create REPL editor")?;
    //if rl.load_history("history.txt").is_err() {
    //    println!("No previous history.");
    //}

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                if line.starts_with('/') {
                    // Route commands to handle_command and pass mutable history
                    let mut empty_history = Vec::new();
                    let result = if let Some(ref mut agent) = agent {
                        handle_command(&line, &mut agent.history)
                    } else {
                        handle_command(&line, &mut empty_history)
                    };

                    match result {
                        Ok(Some(output)) => println!("{}", output),
                        Ok(None) => break, // Exit command
                        Err(e) => eprintln!("Error: {:#?}", e),
                    }
                } else {
                    match handle_line(line, &mut agent).await {
                        Ok(Some(output)) => println!("{}", output),
                        Ok(None) => break, // Exit command
                        Err(e) => eprintln!("Error: {:#?}", e),
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                eprintln!("REPL Error: {:#?}", err);
                break;
            }
        }
    }

    rl.save_history("history.txt")
        .context("Failed to save REPL history")?;

    // Gracefully stop embedded mcp-wallet server on exit
    if let Some(shutdown) = wallet_shutdown {
        shutdown.shutdown();
    }

    Ok(())
}

/// Handles a single line of input from the REPL.
///
/// Returns `Ok(Some(String))` with output, `Ok(None)` to exit, or `Err` on error.
#[allow(dead_code)]
pub async fn handle_line<M: CompletionModel + Send + Sync>(
    line: String,
    agent: &mut Option<ReplAgent<M>>,
) -> Result<Option<String>> {
    if line == "/exit" {
        return Ok(None);
    } else if line == "/help" {
        return Ok(Some("Commands: /exit, /help".to_string()));
    }

    if let Some(ref mut agent) = agent {
        agent.history.push(crate::agent::ChatMessage {
            role: "user".to_string(),
            content: line.clone(),
        });
        match agent.run(&line).await {
            Ok(response) => {
                agent.history.push(crate::agent::ChatMessage {
                    role: "assistant".to_string(),
                    content: response.clone(),
                });
                Ok(Some(format!("Response: {}\n", response)))
            }
            Err(e) => Err(e),
        }
    } else {
        Ok(Some(
            "LLM agent not initialized. Please set GEMINI_API_KEY in your environment or config."
                .to_string(),
        ))
    }
}

/// Handles a command entered by the user.
///
/// This function currently supports a minimal set of commands and is
/// intentionally introduced to accept a mutable reference to the chat
/// history as required by the PRD tasks. Future subtasks will extend it
/// to implement `/show_history` and `/clear_history` behaviors.
///
/// Returns `Ok(Some(String))` with output, `Ok(None)` to signal exit, or
/// `Err` on error.
#[allow(dead_code)]
pub fn handle_command(
    command: &str,
    _history: &mut Vec<crate::agent::ChatMessage>,
) -> Result<Option<String>> {
    match command {
        "/exit" => Ok(None),
        "/help" => Ok(Some(
            "Commands: /exit, /help, /show_history, /clear_history".to_string(),
        )),
        "/show_history" => {
            if _history.is_empty() {
                return Ok(Some("History is empty.".to_string()));
            }

            let mut out = String::new();
            for msg in _history.iter() {
                match msg.role.as_str() {
                    "user" => {
                        out.push_str("> ");
                        out.push_str("User: ");
                    }
                    "assistant" => {
                        out.push_str("< ");
                        out.push_str("Assistant: ");
                    }
                    other => {
                        out.push_str("~ ");
                        out.push_str(other);
                        out.push_str(": ");
                    }
                }
                out.push_str(&msg.content);
                out.push('\n');
            }
            Ok(Some(out))
        }
        "/clear_history" => {
            _history.clear();
            Ok(Some("History cleared.".to_string()))
        }
        _ => Ok(Some(format!("Unknown command: {}", command))),
    }
}
