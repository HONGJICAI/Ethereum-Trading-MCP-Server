mod config;
mod ethereum;
mod mcp;
mod tools;

#[cfg(test)]
mod tests;

use anyhow::Result;
use rmcp::ServiceExt;
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

    // Create MCP server
    let server = mcp::McpServer::new(config).await?;

    // Serve over stdio using tokio stdin/stdout
    info!("Server ready, listening on stdio");
    let transport = (tokio::io::stdin(), tokio::io::stdout());
    server.serve(transport).await?;

    Ok(())
}
