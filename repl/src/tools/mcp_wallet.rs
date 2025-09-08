//! mcp-wallet in-process server bootstrap and transport handle
//!
//! This module starts the mcp-wallet MCP server inside the REPL process using an
//! in-memory duplex stream (stdio-compatible). It returns a handle that can be
//! used to shutdown the server and to obtain the client-side stream for wiring
//! an MCP client/agent in a later step.

use anyhow::{Context, Result};
use mcp_wallet::{eth_client::EthClient, service::WalletHandler, wallet::Wallet, WalletError};
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use rmcp::model::CallToolRequestParam;
use rmcp::service::{RoleClient, RunningService, ServiceExt};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{duplex, DuplexStream};
use tokio::io::{split, ReadHalf, WriteHalf};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::config::Config;

/// Handle to the running in-process MCP wallet server.
pub struct ServerHandle {
    join_handle: JoinHandle<Result<()>>,
    /// The client-side duplex stream to connect an MCP client.
    client_stream: Option<DuplexStream>,
}

impl ServerHandle {
    /// Abort the running server task. Dropping the client stream will also
    /// naturally end the server if it is waiting on I/O.
    pub fn shutdown(self) {
        self.join_handle.abort();
        // client_stream gets dropped here due to move, closing the pipe
    }

    /// Split and take ownership of the client-side IO halves for wiring to an MCP client.
    /// After calling this, the internal client stream is consumed and cannot be retrieved again.
    pub fn into_client_io(
        mut self,
    ) -> (
        (ReadHalf<DuplexStream>, WriteHalf<DuplexStream>),
        ServerShutdown,
    ) {
        let stream = self
            .client_stream
            .take()
            .expect("client_stream already taken");
        let (r, w) = split(stream);
        let shutdown = ServerShutdown {
            join_handle: self.join_handle,
        };
        ((r, w), shutdown)
    }

    /// Take the raw client-side duplex stream (unsplit) for use with serve_client.
    pub fn into_client_stream(mut self) -> (DuplexStream, ServerShutdown) {
        let stream = self
            .client_stream
            .take()
            .expect("client_stream already taken");
        let shutdown = ServerShutdown {
            join_handle: self.join_handle,
        };
        (stream, shutdown)
    }
}

/// Shutdown handle for a running in-process MCP server.
pub struct ServerShutdown {
    join_handle: JoinHandle<Result<()>>,
}

impl ServerShutdown {
    /// Gracefully stop the server by aborting the background task.
    pub fn shutdown(self) {
        self.join_handle.abort();
    }
}

/// Start the mcp-wallet server as an in-process task and return a handle.
///
/// The returned handle contains the client-side duplex stream that carries the
/// stdio-like transport. The agent-side adapter can use this stream to talk to
/// the server (Task 2.2).
pub async fn start_mcp_wallet_server(cfg: &Config) -> Result<ServerHandle> {
    // Resolve wallet file path
    let wallet_path: PathBuf = if let Some(path) = &cfg.wallet_server.wallet_file {
        path.clone()
    } else {
        dirs::home_dir()
            .map(|mut p| {
                p.push(".mcp-wallet.json");
                p
            })
            .ok_or_else(|| WalletError::WalletError("Could not determine home directory".into()))?
    };

    // Load or create wallet
    let mut wallet = match std::fs::read_to_string(&wallet_path) {
        Ok(contents) => {
            tracing::info!(path = %wallet_path.display(), "Loading wallet file");
            serde_json::from_str(&contents).unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to parse wallet file, creating new");
                Wallet::new()
            })
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::info!(path = %wallet_path.display(), "Creating new wallet file");
            Wallet::new()
        }
        Err(e) => {
            return Err(WalletError::FileError(e).into());
        }
    };
    wallet.set_file_path(&wallet_path);

    let wallet = Arc::new(Mutex::new(wallet));
    let eth_client = Arc::new(EthClient::new(&cfg.wallet_server.rpc_url).with_context(|| {
        format!(
            "Failed to create ETH RPC client for {}",
            cfg.wallet_server.rpc_url
        )
    })?);

    let handler = WalletHandler::new(wallet.clone(), eth_client.clone());

    // Create in-memory stdio transport using a duplex stream
    let (server_end, client_end) = duplex(64 * 1024);
    let (server_r, server_w): (ReadHalf<DuplexStream>, WriteHalf<DuplexStream>) = split(server_end);

    let join_handle: JoinHandle<Result<()>> = tokio::spawn(async move {
        tracing::info!("Starting mcp-wallet MCP server (in-process stdio)");
        // Start the server and then wait until it completes
        let running = handler
            .serve((server_r, server_w))
            .await
            .context("failed to start mcp-wallet server")?;
        if let Err(e) = running.waiting().await {
            tracing::error!(error = %e, "mcp-wallet server terminated with error");
            return Err(anyhow::anyhow!(e)).context("mcp-wallet server terminated with error");
        }

        // After the server shuts down, save the wallet if it has changed.
        let wallet = wallet.lock().await;
        if wallet.is_dirty() {
            if let Some(path) = wallet.file_path() {
                tracing::info!(path = %path.display(), "Saving wallet file");
                let contents = serde_json::to_string_pretty(&*wallet)
                    .context("failed to serialize wallet for saving")?;
                std::fs::write(path, contents).with_context(|| {
                    format!("failed to write wallet file to {}", path.display())
                })?;
            }
        }

        Ok(())
    });

    Ok(ServerHandle {
        join_handle,
        client_stream: Some(client_end),
    })
}

// =============================
// Agent Tool: McpWalletTool
// =============================

/// Arguments for the `mcp_wallet` pass-through tool. It forwards to the
/// embedded mcp-wallet server over RMCP using the provided tool name and
/// optional params object.
#[derive(Deserialize, Debug)]
pub struct McpWalletArgs {
    /// Tool name exposed by mcp-wallet (e.g., "new_account", "eth_get_balance").
    pub tool: String,
    /// Optional parameters object forwarded to the MCP tool call.
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Clone)]
pub struct McpWalletTool {
    client: Arc<RunningService<RoleClient, ()>>, // returned by rmcp::serve_client
}

impl McpWalletTool {
    /// Create a new tool from the RMCP client returned by `serve_client`.
    pub fn new(client: RunningService<RoleClient, ()>) -> Self {
        Self {
            client: Arc::new(client),
        }
    }
}

#[derive(Error, Debug)]
pub enum McpWalletToolError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    Service(#[from] rmcp::ServiceError),
}

impl Tool for McpWalletTool {
    const NAME: &'static str = "mcp_wallet";

    type Error = McpWalletToolError;
    type Args = McpWalletArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        // Best-effort: enumerate tools from the server for description.
        let tools_desc = match self.client.list_tools(None).await {
            Ok(list) => {
                let names: Vec<String> = list
                    .tools
                    .into_iter()
                    .map(|t| t.name.into_owned())
                    .collect();
                format!("Available wallet tools: {}", names.join(", "))
            }
            Err(_) => "mcp-wallet tools are available".to_string(),
        };

        ToolDefinition {
            name: Self::NAME.to_string(),
            description: format!(
                "Routes calls to the embedded mcp-wallet MCP server. {}. Provide {{ tool, params }}",
                tools_desc
            ),
            parameters: json!({
                "type": "object",
                "properties": {
                    "tool": { "type": "string", "description": "Tool name exposed by mcp-wallet" },
                    "params": { "type": "object", "description": "Tool parameters object" }
                },
                "required": ["tool"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut map: Option<Map<String, Value>> = None;
        if let Some(val) = args.params {
            match val {
                Value::Object(m) => map = Some(m),
                _ => return Err(anyhow::anyhow!("params must be a JSON object").into()),
            }
        }

        let req = CallToolRequestParam {
            name: args.tool.into(),
            arguments: map,
        };

        let res = self.client.call_tool(req).await?;
        let out = res.structured_content.unwrap_or(Value::Null);
        Ok(out.to_string())
    }
}
