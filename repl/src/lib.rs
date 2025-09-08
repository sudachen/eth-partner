//! The main library for the REPL AI assistant.

#[allow(dead_code)]
pub mod agent;
pub mod config;
pub mod tools;

use crate::agent::ReplAgent;
use crate::config::GenerationConfig;
use crate::tools::web_search::WebSearchTool;
use anyhow::{Context, Result};
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::CompletionModel;
use rig::providers::gemini;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::Editor;
use serde_json::json;

/// Runs the main REPL loop.
#[allow(dead_code)]
pub async fn run_repl() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = config::load().context("Failed to load configuration")?;
    println!("Loaded config: {:?}", config);

    // --- Agent Setup ---
    // Prioritize API key from environment, then fall back to config file.
    let api_key = std::env::var("GEMINI_API_KEY")
        .ok()
        .or(config.llm.google_api_key);

    let agent = if let Some(key) = api_key {
        std::env::set_var("GEMINI_API_KEY", key);
        let client = gemini::Client::from_env();
        println!("Gemini client initialized.");

        let mut agent_builder = client.agent("gemini-1.5-flash-latest").preamble(
            "You are a helpful AI assistant. Be concise and clear. You have access to a set of tools to help you answer questions.",
        );

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
            }
            _ => {
                println!(
                    "Web search tool unavailable: missing GOOGLE_SEARCH_API_KEY and/or GOOGLE_SEARCH_ENGINE_ID."
                );
            }
        }

        Some(ReplAgent::new(agent_builder))
    } else {
        println!(
            "Warning: GEMINI_API_KEY not found in environment or config. LLM functionality will be disabled."
        );
        None
    };

    // --- REPL Loop ---
    let mut rl = Editor::<(), DefaultHistory>::new().context("Failed to create REPL editor")?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                match handle_line(line, &agent).await {
                    Ok(Some(output)) => println!("{}", output),
                    Ok(None) => break, // Exit command
                    Err(e) => eprintln!("Error: {:#?}", e),
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

    Ok(())
}

/// Handles a single line of input from the REPL.
///
/// Returns `Ok(Some(String))` with output, `Ok(None)` to exit, or `Err` on error.
#[allow(dead_code)]
pub async fn handle_line<M: CompletionModel + Send + Sync>(
    line: String,
    agent: &Option<ReplAgent<M>>,
) -> Result<Option<String>> {
    if line == "/exit" {
        return Ok(None);
    } else if line == "/help" {
        return Ok(Some("Commands: /exit, /help".to_string()));
    }

    if let Some(ref agent) = agent {
        match agent.run(&line).await {
            Ok(response) => Ok(Some(format!("Response: {}\n", response))),
            Err(e) => Err(e.into()),
        }
    } else {
        Ok(Some(
            "LLM agent not initialized. Please set GEMINI_API_KEY in your environment or config."
                .to_string(),
        ))
    }
}
