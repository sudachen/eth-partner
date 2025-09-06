//! MCP Wallet Server - Main entry point

use anyhow::Result;
use mcp_wallet::{service::WalletHandler, wallet::Wallet, WalletError};
use rmcp::ServiceExt;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
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
        .ok_or_else(|| {
            WalletError::WalletError("Could not determine home directory".to_string())
        })?;

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

    // Wrap the wallet in an Arc<Mutex<>> to allow shared access
    let wallet = Arc::new(Mutex::new(wallet));

    // Create the wallet service handler
    let handler = WalletHandler::new(wallet.clone());

    // Create the stdio transport
    let transport = (tokio::io::stdin(), tokio::io::stdout());

    // Start the MCP server
    log::info!("MCP Wallet Server started in compliant stdio mode.");
    handler.serve(transport).await?;

    // After the server shuts down, save the wallet if it has changed.
    let wallet = wallet.lock().await;
    if wallet.is_dirty() {
        if let Some(path) = wallet.file_path() {
            log::info!("Saving wallet to {}", path.display());
            let contents = serde_json::to_string_pretty(&*wallet)?;
            std::fs::write(path, contents)?;
        }
    }

    Ok(())
}
