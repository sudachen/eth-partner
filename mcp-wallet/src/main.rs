//! MCP Wallet Server - Main entry point

use anyhow::Result;
use clap::Parser;
use mcp_wallet::{commands::{handle_mcp_command, tool_definition::generate_tool_definition}, wallet::Wallet, WalletError};
use std::io::{self, BufRead};

/// A lightweight, command-line Ethereum wallet server with an MCP interface.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Print the tool definition as JSON and exit.
    #[arg(long)]
    get_tool_definition: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.get_tool_definition {
        let tool_definition = generate_tool_definition();
        let json = serde_json::to_string_pretty(&tool_definition)?;
        println!("{}", json);
        return Ok(());
    }

    // Initialize logger to write to stderr
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    // Determine wallet file path
    let wallet_path = dirs::home_dir()
        .map(|mut path| {
            path.push(".mcp-wallet.json");
            path
        })
        .ok_or_else(|| WalletError::WalletError("Could not determine home directory".to_string()))?;

    // Load or create wallet
    let mut wallet = match std::fs::read_to_string(&wallet_path) {
        Ok(contents) => {
            log::info!("Loading wallet from {}", wallet_path.display());
            serde_json::from_str(&contents).unwrap_or_else(|e| {
                log::warn!("Failed to parse wallet file, creating a new one: {}", e);
                Wallet::new()
            })
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            log::info!("Creating new wallet at {}", wallet_path.display());
            Wallet::new()
        }
        Err(e) => {
            return Err(WalletError::FileError(e).into());
        }
    };

    wallet.set_file_path(&wallet_path);

    // Main MCP server loop
    log::info!("MCP Wallet Server started in stdio mode.");
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                let response = serde_json::json!({ "status": "error", "error": e.to_string() });
                println!("{}", response);
                continue;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        let response = handle_mcp_command(&line, &mut wallet).await;
        let response_json = serde_json::to_string(&response)?;
        println!("{}", response_json);

        // Save wallet if it has been modified
        if wallet.is_dirty() {
            match serde_json::to_string_pretty(&wallet) {
                Ok(wallet_json) => {
                    if let Err(e) = std::fs::write(&wallet_path, wallet_json) {
                        log::error!("Failed to save wallet file: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("Failed to serialize wallet for saving: {}", e);
                }
            }
        }
    }

    Ok(())
}
