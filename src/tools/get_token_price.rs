use super::Tool;
use crate::ethereum::{EthereumClientTrait, UniswapRouterTrait};
use anyhow::{Context, Result};
use async_trait::async_trait;
use ethers::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

// WETH address on Ethereum mainnet
const WETH_ADDRESS: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

// USDC address on Ethereum mainnet (for USD pricing)
const USDC_ADDRESS: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

pub struct GetTokenPriceTool<C: EthereumClientTrait, U: UniswapRouterTrait> {
    client: Arc<C>,
    uniswap: Arc<U>,
}

impl<C: EthereumClientTrait, U: UniswapRouterTrait> GetTokenPriceTool<C, U> {
    pub fn new(client: Arc<C>, uniswap: Arc<U>) -> Self {
        Self { client, uniswap }
    }
}

#[derive(Debug, Deserialize)]
struct GetTokenPriceParams {
    token_address: String,
    #[serde(default = "default_quote_currency")]
    quote_currency: String, // "ETH" or "USD"
}

fn default_quote_currency() -> String {
    "USD".to_string()
}

#[derive(Debug, Serialize)]
struct GetTokenPriceResult {
    token_address: String,
    price: String,
    quote_currency: String,
}

#[async_trait]
impl<C: EthereumClientTrait + 'static, U: UniswapRouterTrait + 'static> Tool
    for GetTokenPriceTool<C, U>
{
    fn name(&self) -> &str {
        "get_token_price"
    }

    fn description(&self) -> &str {
        "Get the current price of a token in USD or ETH using Uniswap V2"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "token_address": {
                    "type": "string",
                    "description": "The token contract address"
                },
                "quote_currency": {
                    "type": "string",
                    "description": "Quote currency: 'USD' or 'ETH' (default: USD)",
                    "enum": ["USD", "ETH"]
                }
            },
            "required": ["token_address"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        let params: GetTokenPriceParams =
            serde_json::from_value(params).context("Invalid parameters for get_token_price")?;

        let token_address: Address = params
            .token_address
            .parse()
            .context("Invalid token address")?;

        // Use 1 token as the base amount (with proper decimals)
        let amount_in = U256::from(10u64.pow(18)); // Assume 18 decimals for simplicity

        let price = if params.quote_currency.to_uppercase() == "ETH" {
            // Get price in WETH
            let weth_address: Address = WETH_ADDRESS.parse().unwrap();
            self.uniswap
                .get_price(token_address, weth_address, amount_in)
                .await?
        } else {
            // Get price in USDC (which represents USD, 6 decimals)
            let usdc_address: Address = USDC_ADDRESS.parse().unwrap();
            let price_ratio = self
                .uniswap
                .get_price(token_address, usdc_address, amount_in)
                .await?;

            // Adjust for USDC having 6 decimals vs assumed 18
            price_ratio * Decimal::from(10u64.pow(12))
        };

        let result = GetTokenPriceResult {
            token_address: params.token_address,
            price: price.to_string(),
            quote_currency: params.quote_currency,
        };

        Ok(serde_json::to_value(result)?)
    }
}
