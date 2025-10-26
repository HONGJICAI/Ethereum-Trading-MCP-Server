use super::Tool;
use crate::ethereum::EthereumClientTrait;
use anyhow::{Context, Result};
use async_trait::async_trait;
use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

pub struct GetBalanceTool<C: EthereumClientTrait> {
    client: Arc<C>,
}

impl<C: EthereumClientTrait> GetBalanceTool<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

#[derive(Debug, Deserialize)]
struct GetBalanceParams {
    address: String,
    token_address: Option<String>,
}

#[derive(Debug, Serialize)]
struct GetBalanceResult {
    address: String,
    balance: String,
    symbol: String,
    decimals: u8,
}

#[async_trait]
impl<C: EthereumClientTrait + 'static> Tool for GetBalanceTool<C> {
    fn name(&self) -> &str {
        "get_balance"
    }

    fn description(&self) -> &str {
        "Query ETH or ERC20 token balance for a given wallet address"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "address": {
                    "type": "string",
                    "description": "The wallet address to query"
                },
                "token_address": {
                    "type": "string",
                    "description": "Optional ERC20 token contract address. If omitted, returns ETH balance"
                }
            },
            "required": ["address"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        let params: GetBalanceParams =
            serde_json::from_value(params).context("Invalid parameters for get_balance")?;

        let address: Address = params.address.parse().context("Invalid wallet address")?;

        let result = if let Some(token_addr_str) = params.token_address {
            // Get ERC20 token balance
            let token_address: Address = token_addr_str.parse().context("Invalid token address")?;

            let (balance, decimals) = self
                .client
                .get_token_balance(token_address, address)
                .await?;
            let symbol = self
                .client
                .get_token_symbol(token_address)
                .await
                .unwrap_or_else(|_| "UNKNOWN".to_string());

            GetBalanceResult {
                address: params.address,
                balance: balance.to_string(),
                symbol,
                decimals,
            }
        } else {
            // Get ETH balance
            let balance = self.client.get_eth_balance(address).await?;

            GetBalanceResult {
                address: params.address,
                balance: balance.to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
            }
        };

        Ok(serde_json::to_value(result)?)
    }
}
