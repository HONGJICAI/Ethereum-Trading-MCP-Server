use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub eth_rpc_url: String,
    pub private_key: String,
    pub chain_id: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let eth_rpc_url = env::var("ETH_RPC_URL")
            .context("ETH_RPC_URL not set in environment")?;
        
        let private_key = env::var("PRIVATE_KEY")
            .context("PRIVATE_KEY not set in environment")?;
        
        let chain_id = env::var("CHAIN_ID")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .context("Invalid CHAIN_ID")?;

        Ok(Self {
            eth_rpc_url,
            private_key,
            chain_id,
        })
    }
}
