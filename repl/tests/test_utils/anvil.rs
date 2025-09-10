use std::process::Stdio;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use rand::Rng;
use tokio::process::{Child, Command};
use tokio::time::sleep;

/// Handle to a spawned Anvil process for tests.
#[allow(dead_code)]
pub struct AnvilHandle {
    pub url: String,
    pub chain_id: u64,
    child: Child,
}

#[allow(dead_code)]
impl AnvilHandle {
    /// Spawn a local anvil instance on a random free port and wait until it's ready.
    pub async fn spawn_and_wait() -> Result<Self> {
        let port: u16 = rand::thread_rng().gen_range(30_000..60_000);
        let url = format!("http://127.0.0.1:{}", port);

        // Use default anvil chain id (31337)
        let chain_id: u64 = 31337;

        // Spawn anvil
        let mut child = Command::new("anvil")
            .arg("--port")
            .arg(port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| "failed to spawn anvil; is it installed and on PATH?")?;

        // Wait until RPC responds
        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_blockNumber",
            "params": []
        });

        const MAX_ATTEMPTS: usize = 50;
        for _ in 0..MAX_ATTEMPTS {
            match client.post(&url).json(&body).send().await {
                Ok(resp) if resp.status().is_success() => {
                    return Ok(Self {
                        url,
                        chain_id,
                        child,
                    });
                }
                _ => sleep(Duration::from_millis(100)).await,
            }
        }

        // If we get here, anvil did not start; kill the process
        let _ = child.kill().await;
        Err(anyhow!("anvil did not become ready at {}", url))
    }

    /// Gracefully stop Anvil.
    pub async fn stop(mut self) -> Result<()> {
        self.child.kill().await.context("failed to kill anvil")?;
        Ok(())
    }
}
