use crate::config::Config;
use crate::ethereum::{EthereumClient, UniswapV2Router};
use crate::tools::{GetBalanceTool, GetTokenPriceTool, SwapTokensTool, Tool as ToolTrait};
use anyhow::{Context, Result};
use rmcp::model::*;
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler};
use rmcp::service::{RequestContext, NotificationContext};
use serde_json::json;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct McpServer {
    get_balance_tool: Arc<GetBalanceTool<EthereumClient>>,
    get_token_price_tool: Arc<GetTokenPriceTool<EthereumClient, UniswapV2Router>>,
    swap_tokens_tool: Arc<SwapTokensTool<EthereumClient, UniswapV2Router>>,
}

impl McpServer {
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Ethereum Trading MCP Server");

        // Initialize Ethereum client
        let client = Arc::new(
            EthereumClient::new(&config.eth_rpc_url, &config.private_key, config.chain_id)
                .await
                .context("Failed to create Ethereum client")?,
        );

        // Initialize Uniswap router
        let uniswap = Arc::new(UniswapV2Router::new(client.get_provider()));

        // Create tool instances
        let get_balance_tool = Arc::new(GetBalanceTool::new(client.clone()));
        let get_token_price_tool = Arc::new(GetTokenPriceTool::new(client.clone(), uniswap.clone()));
        let swap_tokens_tool = Arc::new(SwapTokensTool::new(client.clone(), uniswap.clone()));

        Ok(Self {
            get_balance_tool,
            get_token_price_tool,
            swap_tokens_tool,
        })
    }

    async fn handle_get_balance(&self, params_value: serde_json::Value) -> Result<CallToolResult, String> {
        let result = self.get_balance_tool
            .execute(params_value)
            .await
            .map_err(|e| format!("Failed to get balance: {}", e))?;

        let json_str = serde_json::to_string_pretty(&result)
            .map_err(|e| format!("Failed to serialize result: {}", e))?;

        Ok(CallToolResult::success(vec![Content::text(json_str)]))
    }

    async fn handle_get_token_price(&self, params_value: serde_json::Value) -> Result<CallToolResult, String> {
        let result = self.get_token_price_tool
            .execute(params_value)
            .await
            .map_err(|e| format!("Failed to get token price: {}", e))?;

        let json_str = serde_json::to_string_pretty(&result)
            .map_err(|e| format!("Failed to serialize result: {}", e))?;

        Ok(CallToolResult::success(vec![Content::text(json_str)]))
    }

    async fn handle_swap_tokens(&self, params_value: serde_json::Value) -> Result<CallToolResult, String> {
        let result = self.swap_tokens_tool
            .execute(params_value)
            .await
            .map_err(|e| format!("Failed to simulate swap: {}", e))?;

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

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        info!("Client sent initialize request");
        Ok(self.get_info())
    }

    async fn on_initialized(
        &self,
        _context: NotificationContext<RoleServer>,
    ) {
        info!("Client sent initialized notification - server is ready for requests");
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        info!("list_tools called");
        Ok(ListToolsResult {
            tools: vec![
                Tool {
                    name: self.get_balance_tool.name().to_string().into(),
                    description: Some(self.get_balance_tool.description().to_string().into()),
                    input_schema: Arc::new(
                        self.get_balance_tool.input_schema()
                            .as_object()
                            .unwrap()
                            .clone()
                    ),
                    output_schema: None,
                    annotations: None,
                    title: None,
                    icons: None,
                },
                Tool {
                    name: self.get_token_price_tool.name().to_string().into(),
                    description: Some(self.get_token_price_tool.description().to_string().into()),
                    input_schema: Arc::new(
                        self.get_token_price_tool.input_schema()
                            .as_object()
                            .unwrap()
                            .clone()
                    ),
                    output_schema: None,
                    annotations: None,
                    title: None,
                    icons: None,
                },
                Tool {
                    name: self.swap_tokens_tool.name().to_string().into(),
                    description: Some(self.swap_tokens_tool.description().to_string().into()),
                    input_schema: Arc::new(
                        self.swap_tokens_tool.input_schema()
                            .as_object()
                            .unwrap()
                            .clone()
                    ),
                    output_schema: None,
                    annotations: None,
                    title: None,
                    icons: None,
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
        info!("call_tool called: {}", request.name);
        let args_value = json!(request.arguments.unwrap_or_default());
        
        match request.name.as_ref() {
            "get_balance" => {
                self.handle_get_balance(args_value).await
                    .map_err(|e| McpError::internal_error(e, None))
            }
            "get_token_price" => {
                self.handle_get_token_price(args_value).await
                    .map_err(|e| McpError::internal_error(e, None))
            }
            "swap_tokens" => {
                self.handle_swap_tokens(args_value).await
                    .map_err(|e| McpError::internal_error(e, None))
            }
            _ => Err(McpError::invalid_params(format!("Unknown tool: {}", request.name), None)),
        }
    }
}


