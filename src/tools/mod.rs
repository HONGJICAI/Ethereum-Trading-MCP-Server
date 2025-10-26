mod get_balance;
mod get_token_price;
mod swap_tokens;

#[cfg(test)]
mod tests;

pub use get_balance::GetBalanceTool;
pub use get_token_price::GetTokenPriceTool;
pub use swap_tokens::SwapTokensTool;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    async fn execute(&self, params: Value) -> Result<Value>;
}
