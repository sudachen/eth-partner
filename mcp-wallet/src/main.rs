//! MCP Wallet Server - Main entry point

use anyhow::Result;
use clap::Parser;
use mcp_wallet::{eth_client::EthClient, service::WalletHandler, wallet::Wallet, WalletError};
use rmcp::ServiceExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::{fmt, EnvFilter};

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
    // Initialize logging to write to ./eth-partner-log.txt by default.
    // Forward `log` macros into `tracing` and set a global subscriber with
    // EnvFilter that respects RUST_LOG, defaulting to "info".
    let _ = tracing_log::LogTracer::init();

    let file_appender = tracing_appender::rolling::never(".", "eth-partner-log.txt");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    // Keep guard alive for process lifetime.
    let _guard: &'static _ = Box::leak(Box::new(guard));

    let subscriber = fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .with_writer(non_blocking)
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);

    // Parse command-line arguments
    let args = Args::parse();

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
