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

// Common token addresses on Ethereum mainnet
fn get_token_address_from_symbol(symbol: &str) -> Option<&'static str> {
    match symbol.to_uppercase().as_str() {
        "WETH" => Some(WETH_ADDRESS),
        "USDC" => Some(USDC_ADDRESS),
        "DAI" => Some("0x6B175474E89094C44Da98b954EedeAC495271d0F"),
        "USDT" => Some("0xdAC17F958D2ee523a2206206994597C13D831ec7"),
        "UNI" => Some("0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984"),
        "LINK" => Some("0x514910771AF9Ca656af840dff83E8264EcF986CA"),
        "WBTC" => Some("0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"),
        "AAVE" => Some("0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9"),
        "MKR" => Some("0x9f8F72aA9304c8B593d555F12eF6589cC3A579A2"),
        "SNX" => Some("0xC011a73ee8576Fb46F5E1c5751cA3B9Fe0af2a6F"),
        _ => None,
    }
}

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
    #[serde(default)]
    token_address: Option<String>,
    #[serde(default)]
    token_symbol: Option<String>,
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
        "Get the current price of a token in USD or ETH using Uniswap V2. You can specify the token by address or by symbol (e.g., WETH, USDC, DAI, USDT, UNI, LINK, WBTC, AAVE, MKR, SNX)."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "token_address": {
                    "type": "string",
                    "description": "The token contract address (use either token_address or token_symbol, not both)"
                },
                "token_symbol": {
                    "type": "string",
                    "description": "The token symbol (e.g., WETH, USDC, DAI, USDT, UNI, LINK, WBTC, AAVE, MKR, SNX). Use either token_address or token_symbol, not both."
                },
                "quote_currency": {
                    "type": "string",
                    "description": "Quote currency: 'USD' or 'ETH' (default: USD)",
                    "enum": ["USD", "ETH"]
                }
            },
            "oneOf": [
                {"required": ["token_address"]},
                {"required": ["token_symbol"]}
            ]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        let params: GetTokenPriceParams =
            serde_json::from_value(params).context("Invalid parameters for get_token_price")?;

        // Determine the token address from either the address or symbol
        let token_address_str = if let Some(addr) = params.token_address {
            addr
        } else if let Some(symbol) = params.token_symbol {
            get_token_address_from_symbol(&symbol)
                .ok_or_else(|| {
                    anyhow::anyhow!("Unknown token symbol: {}. Supported symbols: WETH, USDC, DAI, USDT, UNI, LINK, WBTC, AAVE, MKR, SNX", symbol)
                })?
                .to_string()
        } else {
            return Err(anyhow::anyhow!(
                "Either token_address or token_symbol must be provided"
            ));
        };

        let token_address: Address = token_address_str
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
            token_address: token_address_str,
            price: price.to_string(),
            quote_currency: params.quote_currency,
        };

        Ok(serde_json::to_value(result)?)
    }
}
