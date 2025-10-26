use anyhow::{Context, Result};
use ethers::prelude::*;
use std::sync::Arc;
use rust_decimal::Decimal;
use std::str::FromStr;

// Uniswap V2 Router address on Ethereum mainnet
const UNISWAP_V2_ROUTER: &str = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D";

pub struct UniswapV2Router {
    provider: Arc<Provider<Http>>,
    router_address: Address,
}

impl UniswapV2Router {
    pub fn new(provider: Arc<Provider<Http>>) -> Self {
        let router_address = UNISWAP_V2_ROUTER.parse().unwrap();
        Self {
            provider,
            router_address,
        }
    }

    /// Simulate a token swap and return expected output amount
    pub async fn simulate_swap(
        &self,
        from_token: Address,
        to_token: Address,
        amount_in: U256,
        wallet_address: Address,
    ) -> Result<SwapSimulation> {
        abigen!(
            IUniswapV2Router02,
            r#"[
                function getAmountsOut(uint amountIn, address[] memory path) external view returns (uint[] memory amounts)
                function swapExactTokensForTokens(uint amountIn, uint amountOutMin, address[] calldata path, address to, uint deadline) external returns (uint[] memory amounts)
            ]"#
        );

        let router = IUniswapV2Router02::new(self.router_address, Arc::clone(&self.provider));

        // Get amounts out for the swap path
        let path = vec![from_token, to_token];
        let amounts = router
            .get_amounts_out(amount_in, path.clone())
            .call()
            .await
            .context("Failed to get amounts out from Uniswap")?;

        let amount_out = amounts.get(1).copied().unwrap_or(U256::zero());

        // Simulate the actual swap transaction using eth_call
        let deadline = U256::from(u64::MAX); // Use max for simulation
        let amount_out_min = U256::zero(); // No slippage protection for simulation

        // Build the transaction
        let swap_call = router.swap_exact_tokens_for_tokens(
            amount_in,
            amount_out_min,
            path,
            wallet_address,
            deadline,
        );

        // Estimate gas
        let gas_estimate = swap_call
            .estimate_gas()
            .await
            .unwrap_or(U256::from(200000)); // Default gas estimate

        // Get current gas price
        let gas_price = self.provider
            .get_gas_price()
            .await
            .unwrap_or(U256::from(50_000_000_000u64)); // 50 gwei default

        let gas_cost = gas_estimate * gas_price;

        Ok(SwapSimulation {
            amount_in,
            amount_out,
            gas_estimate,
            gas_price,
            gas_cost,
        })
    }

    /// Get the best price for a token pair
    pub async fn get_price(
        &self,
        from_token: Address,
        to_token: Address,
        amount_in: U256,
    ) -> Result<Decimal> {
        abigen!(
            IUniswapV2Router02,
            r#"[
                function getAmountsOut(uint amountIn, address[] memory path) external view returns (uint[] memory amounts)
            ]"#
        );

        let router = IUniswapV2Router02::new(self.router_address, Arc::clone(&self.provider));

        let path = vec![from_token, to_token];
        let amounts = router
            .get_amounts_out(amount_in, path)
            .call()
            .await
            .context("Failed to get price from Uniswap")?;

        let amount_out = amounts.get(1).copied().unwrap_or(U256::zero());

        // Calculate price ratio
        let amount_in_decimal = Decimal::from_str(&amount_in.to_string())?;
        let amount_out_decimal = Decimal::from_str(&amount_out.to_string())?;
        
        let price = if amount_in_decimal.is_zero() {
            Decimal::ZERO
        } else {
            amount_out_decimal / amount_in_decimal
        };

        Ok(price)
    }
}

#[derive(Debug, Clone)]
pub struct SwapSimulation {
    pub amount_in: U256,
    pub amount_out: U256,
    pub gas_estimate: U256,
    pub gas_price: U256,
    pub gas_cost: U256,
}
