use super::Tool;
use crate::ethereum::{EthereumClientTrait, UniswapRouterTrait};
use anyhow::{Context, Result};
use async_trait::async_trait;
use ethers::prelude::*;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::str::FromStr;
use std::sync::Arc;

pub struct SwapTokensTool<C: EthereumClientTrait, U: UniswapRouterTrait> {
    client: Arc<C>,
    uniswap: Arc<U>,
}

impl<C: EthereumClientTrait, U: UniswapRouterTrait> SwapTokensTool<C, U> {
    pub fn new(client: Arc<C>, uniswap: Arc<U>) -> Self {
        Self { client, uniswap }
    }
}

#[derive(Debug, Deserialize)]
struct SwapTokensParams {
    from_token: String,
    to_token: String,
    amount: String,
    #[serde(default = "default_slippage")]
    slippage_tolerance: f64, // Percentage (e.g., 0.5 for 0.5%)
}

fn default_slippage() -> f64 {
    0.5
}

#[derive(Debug, Serialize)]
struct SwapTokensResult {
    from_token: String,
    to_token: String,
    amount_in: String,
    estimated_amount_out: String,
    minimum_amount_out: String,
    gas_estimate: String,
    gas_price_gwei: String,
    estimated_gas_cost_eth: String,
    slippage_tolerance: f64,
}

#[async_trait]
impl<C: EthereumClientTrait + 'static, U: UniswapRouterTrait + 'static> Tool
    for SwapTokensTool<C, U>
{
    fn name(&self) -> &str {
        "swap_tokens"
    }

    fn description(&self) -> &str {
        "Simulate a token swap on Uniswap V2. Returns estimated output and gas costs without executing the transaction."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "from_token": {
                    "type": "string",
                    "description": "Address of the token to swap from"
                },
                "to_token": {
                    "type": "string",
                    "description": "Address of the token to swap to"
                },
                "amount": {
                    "type": "string",
                    "description": "Amount to swap (in human-readable format, e.g., '1.5' for 1.5 tokens)"
                },
                "slippage_tolerance": {
                    "type": "number",
                    "description": "Slippage tolerance in percentage (default: 0.5)"
                }
            },
            "required": ["from_token", "to_token", "amount"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        let params: SwapTokensParams =
            serde_json::from_value(params).context("Invalid parameters for swap_tokens")?;

        let from_token: Address = params
            .from_token
            .parse()
            .context("Invalid from_token address")?;

        let to_token: Address = params
            .to_token
            .parse()
            .context("Invalid to_token address")?;

        // Parse amount - assume 18 decimals for simplicity
        // In production, you'd query the token's decimals
        let amount_decimal = Decimal::from_str(&params.amount).context("Invalid amount")?;
        let amount_wei = amount_decimal * Decimal::from(10u64.pow(18));
        // Round to remove any decimal places and convert to integer string
        let amount_wei_rounded = amount_wei.round();
        let amount_in = U256::from_dec_str(&amount_wei_rounded.to_string())
            .context("Failed to convert amount to U256")?;

        // Get wallet address
        let wallet_address = self.client.get_wallet_address();

        // Simulate the swap
        let simulation = self
            .uniswap
            .simulate_swap(from_token, to_token, amount_in, wallet_address)
            .await?;

        // Calculate minimum amount out with slippage
        let slippage_multiplier = 1.0 - (params.slippage_tolerance / 100.0);
        let amount_out_decimal = Decimal::from_str(&simulation.amount_out.to_string())?;
        let min_amount_out =
            amount_out_decimal * Decimal::from_f64(slippage_multiplier).unwrap_or(Decimal::ONE);

        // Convert amounts to human-readable format (assuming 18 decimals)
        let estimated_out = amount_out_decimal / Decimal::from(10u64.pow(18));
        let minimum_out = min_amount_out / Decimal::from(10u64.pow(18));

        // Convert gas price to Gwei
        let gas_price_gwei =
            Decimal::from_str(&simulation.gas_price.to_string())? / Decimal::from(10u64.pow(9));

        // Convert gas cost to ETH
        let gas_cost_eth =
            Decimal::from_str(&simulation.gas_cost.to_string())? / Decimal::from(10u64.pow(18));

        let result = SwapTokensResult {
            from_token: params.from_token,
            to_token: params.to_token,
            amount_in: params.amount,
            estimated_amount_out: estimated_out.to_string(),
            minimum_amount_out: minimum_out.to_string(),
            gas_estimate: simulation.gas_estimate.to_string(),
            gas_price_gwei: gas_price_gwei.to_string(),
            estimated_gas_cost_eth: gas_cost_eth.to_string(),
            slippage_tolerance: params.slippage_tolerance,
        };

        Ok(serde_json::to_value(result)?)
    }
}
