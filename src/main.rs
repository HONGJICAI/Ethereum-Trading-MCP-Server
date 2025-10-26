mod config;
mod ethereum;
mod mcp;
mod tools;

#[cfg(test)]
mod tests;

use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging - MUST write to stderr, not stdout!
    // stdout is reserved for JSON-RPC protocol messages
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_writer(std::io::stderr)
        .init();

    info!("Starting Ethereum Trading MCP Server");

    // Load configuration
    dotenv::dotenv().ok();
    let config = config::Config::from_env()?;

    // Create MCP server
    let server = mcp::McpServer::new(config).await?;

    // Serve over stdio using tokio stdin/stdout
    info!("Server ready, listening on stdio");
    let service = server.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    service.waiting().await?;

    Ok(())
}
