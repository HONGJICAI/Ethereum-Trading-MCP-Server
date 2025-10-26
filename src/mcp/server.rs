use crate::config::Config;
use crate::ethereum::{EthereumClient, UniswapV2Router};
use crate::tools::{GetBalanceTool, GetTokenPriceTool, SwapTokensTool, Tool};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use tracing::{error, info};

pub struct McpServer {
    tools: HashMap<String, Box<dyn Tool>>,
    server_info: ServerInfo,
}

#[derive(Debug, Serialize)]
struct ServerInfo {
    name: String,
    version: String,
    protocol_version: String,
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl McpServer {
    pub async fn new(config: Config) -> Result<Self> {
        // Initialize Ethereum client
        let client = Arc::new(
            EthereumClient::new(&config.eth_rpc_url, &config.private_key, config.chain_id)
                .await
                .context("Failed to create Ethereum client")?,
        );

        // Initialize Uniswap router
        let uniswap = Arc::new(UniswapV2Router::new(client.get_provider()));

        // Register tools
        let mut tools: HashMap<String, Box<dyn Tool>> = HashMap::new();

        let get_balance = Box::new(GetBalanceTool::new(Arc::clone(&client)));
        tools.insert(get_balance.name().to_string(), get_balance);

        let get_token_price = Box::new(GetTokenPriceTool::new(Arc::clone(&client), Arc::clone(&uniswap)));
        tools.insert(get_token_price.name().to_string(), get_token_price);

        let swap_tokens = Box::new(SwapTokensTool::new(Arc::clone(&client), Arc::clone(&uniswap)));
        tools.insert(swap_tokens.name().to_string(), swap_tokens);

        let server_info = ServerInfo {
            name: "Ethereum Trading MCP Server".to_string(),
            version: "0.1.0".to_string(),
            protocol_version: "2024-11-05".to_string(),
        };

        Ok(Self { tools, server_info })
    }

    pub async fn run(&self) -> Result<()> {
        info!("MCP Server started, waiting for requests on stdin");

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = line.context("Failed to read line from stdin")?;
            
            if line.trim().is_empty() {
                continue;
            }

            let response = self.handle_request(&line).await;
            let response_json = serde_json::to_string(&response)?;
            
            writeln!(stdout, "{}", response_json)?;
            stdout.flush()?;
        }

        Ok(())
    }

    async fn handle_request(&self, request_str: &str) -> JsonRpcResponse {
        // Parse JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_str(request_str) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse JSON-RPC request: {}", e);
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: "Parse error".to_string(),
                        data: Some(json!({"details": e.to_string()})),
                    }),
                };
            }
        };

        info!("Received request: method={}", request.method);

        // Handle different MCP methods
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(),
            "tools/list" => self.handle_list_tools(),
            "tools/call" => self.handle_tool_call(request.params).await,
            _ => Err(anyhow::anyhow!("Unknown method: {}", request.method)),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(e) => {
                error!("Error handling request: {}", e);
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: "Internal error".to_string(),
                        data: Some(json!({"details": e.to_string()})),
                    }),
                }
            }
        }
    }

    fn handle_initialize(&self) -> Result<Value> {
        Ok(json!({
            "protocolVersion": self.server_info.protocol_version,
            "serverInfo": {
                "name": self.server_info.name,
                "version": self.server_info.version,
            },
            "capabilities": {
                "tools": {}
            }
        }))
    }

    fn handle_list_tools(&self) -> Result<Value> {
        let tools: Vec<Value> = self
            .tools
            .values()
            .map(|tool| {
                json!({
                    "name": tool.name(),
                    "description": tool.description(),
                    "inputSchema": tool.input_schema(),
                })
            })
            .collect();

        Ok(json!({ "tools": tools }))
    }

    async fn handle_tool_call(&self, params: Option<Value>) -> Result<Value> {
        let params = params.context("Missing parameters for tools/call")?;
        
        let tool_name = params["name"]
            .as_str()
            .context("Missing or invalid 'name' in tool call")?;
        
        let arguments = params["arguments"].clone();

        let tool = self
            .tools
            .get(tool_name)
            .context(format!("Unknown tool: {}", tool_name))?;

        info!("Executing tool: {}", tool_name);
        let result = tool.execute(arguments).await?;

        Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&result)?
            }]
        }))
    }
}

