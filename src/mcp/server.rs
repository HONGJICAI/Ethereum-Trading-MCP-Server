use crate::config::Config;
use crate::ethereum::{EthereumClient, EthereumClientTrait, UniswapV2Router};
use anyhow::{Context, Result};
use async_trait::async_trait;
use ethers::prelude::*;
use rmcp::model::*;
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler};
use rmcp::service::RequestContext;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[derive(Clone)]
pub struct McpServer {
    client: Arc<RwLock<EthereumClient>>,
    uniswap: Arc<RwLock<UniswapV2Router>>,
}

// WETH address on Ethereum mainnet
const WETH_ADDRESS: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

// USDC address on Ethereum mainnet (for USD pricing)
const USDC_ADDRESS: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct GetBalanceParams {
    address: String,
    token_address: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
struct GetBalanceResult {
    address: String,
    balance: String,
    symbol: String,
    decimals: u8,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct GetTokenPriceParams {
    token_address: String,
    #[serde(default = "default_quote_currency")]
    quote_currency: String,
}

fn default_quote_currency() -> String {
    "USD".to_string()
}

#[derive(Debug, Serialize, JsonSchema)]
struct GetTokenPriceResult {
    token_address: String,
    price: String,
    quote_currency: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct SwapTokensParams {
    from_token: String,
    to_token: String,
    amount: String,
    #[serde(default = "default_slippage")]
    slippage_tolerance: f64,
}

fn default_slippage() -> f64 {
    0.5
}

#[derive(Debug, Serialize, JsonSchema)]
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

impl McpServer {
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Ethereum Trading MCP Server");

        // Initialize Ethereum client
        let client = Arc::new(RwLock::new(
            EthereumClient::new(&config.eth_rpc_url, &config.private_key, config.chain_id)
                .await
                .context("Failed to create Ethereum client")?,
        ));

        // Initialize Uniswap router
        let uniswap = Arc::new(RwLock::new(UniswapV2Router::new(
            client.read().await.get_provider(),
        )));

        Ok(Self { client, uniswap })
    }

    async fn handle_get_balance(&self, params: GetBalanceParams) -> Result<CallToolResult, String> {
        let address: Address = params
            .address
            .parse()
            .map_err(|e| format!("Invalid wallet address: {}", e))?;

        let client = self.client.read().await;

        let result = if let Some(token_addr_str) = params.token_address {
            // Get ERC20 token balance
            let token_address: Address = token_addr_str
                .parse()
                .map_err(|e| format!("Invalid token address: {}", e))?;

            let (balance, decimals) = client
                .get_token_balance(token_address, address)
                .await
                .map_err(|e| format!("Failed to get token balance: {}", e))?;

            let symbol = client
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
            let balance = client
                .get_eth_balance(address)
                .await
                .map_err(|e| format!("Failed to get ETH balance: {}", e))?;

            GetBalanceResult {
                address: params.address,
                balance: balance.to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
            }
        };

        let json_str = serde_json::to_string_pretty(&result)
            .map_err(|e| format!("Failed to serialize result: {}", e))?;

        Ok(CallToolResult::success(vec![Content::text(json_str)]))
    }

    async fn handle_get_token_price(&self, params: GetTokenPriceParams) -> Result<CallToolResult, String> {
        let token_address: Address = params
            .token_address
            .parse()
            .map_err(|e| format!("Invalid token address: {}", e))?;

        let amount_in = U256::from(10u64.pow(18));

        let uniswap = self.uniswap.read().await;

        let price = if params.quote_currency.to_uppercase() == "ETH" {
            let weth_address: Address = WETH_ADDRESS.parse().unwrap();
            uniswap
                .get_price(token_address, weth_address, amount_in)
                .await
                .map_err(|e| format!("Failed to get price: {}", e))?
        } else {
            let usdc_address: Address = USDC_ADDRESS.parse().unwrap();
            let price_ratio = uniswap
                .get_price(token_address, usdc_address, amount_in)
                .await
                .map_err(|e| format!("Failed to get price: {}", e))?;

            price_ratio * Decimal::from(10u64.pow(12))
        };

        let result = GetTokenPriceResult {
            token_address: params.token_address,
            price: price.to_string(),
            quote_currency: params.quote_currency,
        };

        let json_str = serde_json::to_string_pretty(&result)
            .map_err(|e| format!("Failed to serialize result: {}", e))?;

        Ok(CallToolResult::success(vec![Content::text(json_str)]))
    }

    async fn handle_swap_tokens(&self, params: SwapTokensParams) -> Result<CallToolResult, String> {
        let from_token: Address = params
            .from_token
            .parse()
            .map_err(|e| format!("Invalid from_token address: {}", e))?;

        let to_token: Address = params
            .to_token
            .parse()
            .map_err(|e| format!("Invalid to_token address: {}", e))?;

        let amount_decimal = Decimal::from_str(&params.amount)
            .map_err(|e| format!("Invalid amount: {}", e))?;
        let amount_wei = amount_decimal * Decimal::from(10u64.pow(18));
        let amount_wei_rounded = amount_wei.round();
        let amount_in = U256::from_dec_str(&amount_wei_rounded.to_string())
            .map_err(|e| format!("Failed to convert amount: {}", e))?;

        let client = self.client.read().await;
        let wallet_address = client.get_wallet_address();

        let uniswap = self.uniswap.read().await;
        let simulation = uniswap
            .simulate_swap(from_token, to_token, amount_in, wallet_address)
            .await
            .map_err(|e| format!("Failed to simulate swap: {}", e))?;

        let slippage_multiplier = 1.0 - (params.slippage_tolerance / 100.0);
        let amount_out_decimal = Decimal::from_str(&simulation.amount_out.to_string())
            .map_err(|e| format!("Failed to parse amount: {}", e))?;
        let min_amount_out =
            amount_out_decimal * Decimal::from_f64(slippage_multiplier).unwrap_or(Decimal::ONE);

        let estimated_out = amount_out_decimal / Decimal::from(10u64.pow(18));
        let minimum_out = min_amount_out / Decimal::from(10u64.pow(18));

        let gas_price_gwei = Decimal::from_str(&simulation.gas_price.to_string())
            .map_err(|e| format!("Failed to parse gas price: {}", e))?
            / Decimal::from(10u64.pow(9));

        let gas_cost_eth = Decimal::from_str(&simulation.gas_cost.to_string())
            .map_err(|e| format!("Failed to parse gas cost: {}", e))?
            / Decimal::from(10u64.pow(18));

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

        let json_str = serde_json::to_string_pretty(&result)
            .map_err(|e| format!("Failed to serialize result: {}", e))?;

        Ok(CallToolResult::success(vec![Content::text(json_str)]))
    }
}

impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Ethereum Trading MCP Server - Provides tools for querying balances, getting token prices, and simulating swaps on Ethereum".to_string()),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult {
            tools: vec![
                Tool {
                    name: "get_balance".into(),
                    description: Some("Query ETH or ERC20 token balance for a given wallet address".into()),
                    input_schema: Arc::new(
                        serde_json::to_value(&schemars::schema_for!(GetBalanceParams))
                            .unwrap()
                            .as_object()
                            .unwrap()
                            .clone()
                    ),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "get_token_price".into(),
                    description: Some("Get the current price of a token in USD or ETH using Uniswap V2".into()),
                    input_schema: Arc::new(
                        serde_json::to_value(&schemars::schema_for!(GetTokenPriceParams))
                            .unwrap()
                            .as_object()
                            .unwrap()
                            .clone()
                    ),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "swap_tokens".into(),
                    description: Some("Simulate a token swap on Uniswap V2. Returns estimated output and gas costs without executing the transaction.".into()),
                    input_schema: Arc::new(
                        serde_json::to_value(&schemars::schema_for!(SwapTokensParams))
                            .unwrap()
                            .as_object()
                            .unwrap()
                            .clone()
                    ),
                    output_schema: None,
                    annotations: None,
                },
            ],
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let args_value = json!(request.arguments.unwrap_or_default());
        
        match request.name.as_ref() {
            "get_balance" => {
                let params: GetBalanceParams = serde_json::from_value(args_value)
                    .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?;
                self.handle_get_balance(params).await
                    .map_err(|e| McpError::internal_error(e, None))
            }
            "get_token_price" => {
                let params: GetTokenPriceParams = serde_json::from_value(args_value)
                    .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?;
                self.handle_get_token_price(params).await
                    .map_err(|e| McpError::internal_error(e, None))
            }
            "swap_tokens" => {
                let params: SwapTokensParams = serde_json::from_value(args_value)
                    .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?;
                self.handle_swap_tokens(params).await
                    .map_err(|e| McpError::internal_error(e, None))
            }
            _ => Err(McpError::invalid_params(format!("Unknown tool: {}", request.name), None)),
        }
    }
}


