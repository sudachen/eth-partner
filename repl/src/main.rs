mod config;
mod agent;
mod tools;

use crate::agent::ReplAgent;
use crate::tools::web_search::WebSearchTool;
use anyhow::{Context, Result};
use rig::client::{CompletionClient, ProviderClient};
use rig::providers::gemini;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::Editor;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::load().context("Failed to load configuration")?;
    println!("Loaded config: {:?}", config);

    // --- Agent Setup ---
    let agent = if let Some(api_key) = config.llm.google_api_key {
        std::env::set_var("GEMINI_API_KEY", api_key);
        let client = gemini::Client::from_env();
        println!("Gemini client initialized.");

        let mut agent_builder = client
            .agent("gemini-1.5-flash-latest")
            .preamble("You are a helpful AI assistant. Be concise and clear. You have access to a set of tools to help you answer questions.");

        if let Some(brave_api_key) = config.tools.brave_api_key {
            agent_builder = agent_builder.tool(WebSearchTool::new(brave_api_key));
            println!("Web search tool initialized.");
        }

        Some(ReplAgent::new(agent_builder))
    } else {
        println!("Warning: GEMINI_API_KEY not found in config. LLM functionality will be disabled.");
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
                if line == "/exit" {
                    break;
                } else if line == "/help" {
                    println!("Commands: /exit, /help");
                } else if let Some(ref agent) = agent {
                    match agent.run(&line).await {
                        Ok(response) => println!("Response: {}\n", response),
                        Err(e) => eprintln!("Agent error: {:#?}\n", e),
                    }
                } else {
                    println!("LLM agent not initialized. Please set GEMINI_API_KEY in your config.");
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

    rl.save_history("history.txt").context("Failed to save REPL history")?;

    Ok(())
}
