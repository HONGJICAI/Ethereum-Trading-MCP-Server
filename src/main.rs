mod config;
mod ethereum;
mod mcp;
mod tools;

#[cfg(test)]
mod tests;

use anyhow::Result;
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting Ethereum Trading MCP Server");

    // Load configuration
    dotenv::dotenv().ok();
    let config = config::Config::from_env()?;

    // Start MCP server
    let server = mcp::McpServer::new(config).await?;
    server.run().await?;

    Ok(())
}
