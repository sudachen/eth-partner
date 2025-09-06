//! MCP Wallet Server - Main entry point

use anyhow::Result;
use clap::Parser;
use mcp_wallet::{
    eth_client::EthClient, service::WalletHandler, wallet::Wallet, wallet_storage, WalletError,
};
use rmcp::ServiceExt;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Command-line arguments for the MCP Wallet Server.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The URL of the Ethereum RPC endpoint.
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger to write to stderr
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    // Parse command-line arguments
    let args = Args::parse();

    // Determine home directory paths
    let home_dir = dirs::home_dir()
        .ok_or_else(|| WalletError::WalletError("Could not determine home directory".to_string()))?;

    let wallet_path = home_dir.join(".mcp-wallet.json");
    let key_path = home_dir.join(".mcp-wallet/key.json");

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

    // Load the private key if it exists
    if key_path.exists() {
        match wallet_storage::load_key(&key_path) {
            Ok(signer) => {
                log::info!("Loaded private key from {}", key_path.display());
                wallet.set_signer(signer);
            }
            Err(e) => {
                log::warn!("Failed to load private key from {}: {}", key_path.display(), e);
            }
        }
    } else {
        log::warn!("Key file not found at {}. Signing will not be possible.", key_path.display());
    }

    wallet.set_file_path(&wallet_path);

    // Wrap the wallet in an Arc<Mutex<>> to allow shared access
    let wallet = Arc::new(Mutex::new(wallet));

    // Create the Ethereum RPC client
    let eth_client = Arc::new(EthClient::new(&args.rpc_url)?);

    // Create the wallet service handler
    let handler = WalletHandler::new(wallet.clone(), eth_client.clone());

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
