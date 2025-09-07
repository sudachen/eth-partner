mod agent;
mod config;
mod tools;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    repl::run_repl().await
}
