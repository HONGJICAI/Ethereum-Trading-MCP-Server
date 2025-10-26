use crate::ethereum::{MockEthereumClient, MockUniswapRouter, SwapSimulation};
use crate::tools::*;
use ethers::prelude::*;
use rust_decimal::Decimal;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_get_balance_tool_with_mock() {
    // Setup mock client with test data
    let wallet_addr: Address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
        .parse()
        .unwrap();
    let mock_client = MockEthereumClient::new()
        .with_wallet_address(wallet_addr)
        .with_eth_balance(wallet_addr, Decimal::new(5, 0)); // 5 ETH

    // Create tool with mock client
    let tool = GetBalanceTool::new(Arc::new(mock_client));

    // Execute the tool
    let params = json!({
        "address": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
    });

    let result = tool.execute(params).await.unwrap();

    // Verify the result
    assert_eq!(result["balance"], "5");
    assert_eq!(result["symbol"], "ETH");
    assert_eq!(result["decimals"], 18);
}

#[tokio::test]
async fn test_get_balance_tool_with_token() {
    // Setup mock client with token balance
    let wallet_addr: Address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
        .parse()
        .unwrap();
    let token_addr: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        .parse()
        .unwrap(); // USDC

    let mock_client = MockEthereumClient::new()
        .with_wallet_address(wallet_addr)
        .with_token_balance(token_addr, wallet_addr, Decimal::new(1000, 0), 6)
        .with_token_symbol(token_addr, "USDC".to_string());

    // Create tool with mock client
    let tool = GetBalanceTool::new(Arc::new(mock_client));

    // Execute the tool
    let params = json!({
        "address": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        "token_address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
    });

    let result = tool.execute(params).await.unwrap();

    // Verify the result
    assert_eq!(result["balance"], "1000");
    assert_eq!(result["symbol"], "USDC");
    assert_eq!(result["decimals"], 6);
}

#[tokio::test]
async fn test_get_token_price_tool_with_mock() {
    // Setup mock clients
    let wallet_addr: Address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
        .parse()
        .unwrap();
    let token_addr: Address = "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984"
        .parse()
        .unwrap(); // UNI
    let usdc_addr: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        .parse()
        .unwrap();

    let mock_client = MockEthereumClient::new().with_wallet_address(wallet_addr);

    // Price of 1 UNI = 10 USDC (adjusted for decimals)
    let mock_uniswap =
        MockUniswapRouter::new().with_price(token_addr, usdc_addr, Decimal::new(10, 0));

    // Create tool with mocks
    let tool = GetTokenPriceTool::new(Arc::new(mock_client), Arc::new(mock_uniswap));

    // Execute the tool
    let params = json!({
        "token_address": "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984",
        "quote_currency": "USD"
    });

    let result = tool.execute(params).await.unwrap();

    // Verify the result
    assert_eq!(
        result["token_address"],
        "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984"
    );
    assert_eq!(result["quote_currency"], "USD");
    // Price should be multiplied by 10^12 for USDC decimal adjustment
    assert!(result["price"].as_str().unwrap().contains("10000000000000"));
}

#[tokio::test]
async fn test_swap_tokens_tool_with_mock() {
    // Setup mock clients
    let wallet_addr: Address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
        .parse()
        .unwrap();
    let from_token: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        .parse()
        .unwrap(); // USDC
    let to_token: Address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
        .parse()
        .unwrap(); // WETH

    let mock_client = MockEthereumClient::new().with_wallet_address(wallet_addr);

    // Create a swap simulation
    let simulation = SwapSimulation {
        amount_in: U256::from_dec_str("1000000000000000000").unwrap(), // 1 token
        amount_out: U256::from_dec_str("500000000000000000").unwrap(), // 0.5 tokens out
        gas_estimate: U256::from(200000),
        gas_price: U256::from(50_000_000_000u64), // 50 gwei
        gas_cost: U256::from(10_000_000_000_000_000u64), // 0.01 ETH
    };

    let mock_uniswap =
        MockUniswapRouter::new().with_swap_simulation(from_token, to_token, simulation);

    // Create tool with mocks
    let tool = SwapTokensTool::new(Arc::new(mock_client), Arc::new(mock_uniswap));

    // Execute the tool
    let params = json!({
        "from_token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "to_token": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        "amount": "1.0",
        "slippage_tolerance": 0.5
    });

    let result = tool.execute(params).await.unwrap();

    // Verify the result
    assert_eq!(
        result["from_token"],
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
    );
    assert_eq!(
        result["to_token"],
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
    );
    assert_eq!(result["amount_in"], "1.0");
    // The decimal formatting may vary, so check it contains the expected value
    let estimated_out = result["estimated_amount_out"].as_str().unwrap();
    assert!(estimated_out == "0.5" || estimated_out == "0.50");
    assert_eq!(result["gas_estimate"], "200000");
}

// Tests for Tool trait methods: name, description, input_schema

#[test]
fn test_get_balance_tool_name() {
    let wallet_addr: Address = Address::zero();
    let mock_client = MockEthereumClient::new().with_wallet_address(wallet_addr);
    let tool = GetBalanceTool::new(Arc::new(mock_client));

    assert_eq!(tool.name(), "get_balance");
}

#[test]
fn test_get_balance_tool_description() {
    let wallet_addr: Address = Address::zero();
    let mock_client = MockEthereumClient::new().with_wallet_address(wallet_addr);
    let tool = GetBalanceTool::new(Arc::new(mock_client));

    let description = tool.description();
    assert!(!description.is_empty());
    assert!(description.contains("balance") || description.contains("Balance"));
}

#[test]
fn test_get_balance_tool_input_schema() {
    let wallet_addr: Address = Address::zero();
    let mock_client = MockEthereumClient::new().with_wallet_address(wallet_addr);
    let tool = GetBalanceTool::new(Arc::new(mock_client));

    let schema = tool.input_schema();

    // Verify it's an object type
    assert_eq!(schema["type"], "object");

    // Verify it has properties
    assert!(schema["properties"].is_object());
    assert!(schema["properties"]["address"].is_object());

    // Verify required fields
    assert!(schema["required"].is_array());
    let required = schema["required"].as_array().unwrap();
    assert!(required.contains(&json!("address")));
}

#[test]
fn test_get_token_price_tool_name() {
    let wallet_addr: Address = Address::zero();
    let mock_client = MockEthereumClient::new().with_wallet_address(wallet_addr);
    let mock_uniswap = MockUniswapRouter::new();
    let tool = GetTokenPriceTool::new(Arc::new(mock_client), Arc::new(mock_uniswap));

    assert_eq!(tool.name(), "get_token_price");
}

#[test]
fn test_get_token_price_tool_description() {
    let wallet_addr: Address = Address::zero();
    let mock_client = MockEthereumClient::new().with_wallet_address(wallet_addr);
    let mock_uniswap = MockUniswapRouter::new();
    let tool = GetTokenPriceTool::new(Arc::new(mock_client), Arc::new(mock_uniswap));

    let description = tool.description();
    assert!(!description.is_empty());
    assert!(description.contains("price") || description.contains("Price"));
}

#[test]
fn test_get_token_price_tool_input_schema() {
    let wallet_addr: Address = Address::zero();
    let mock_client = MockEthereumClient::new().with_wallet_address(wallet_addr);
    let mock_uniswap = MockUniswapRouter::new();
    let tool = GetTokenPriceTool::new(Arc::new(mock_client), Arc::new(mock_uniswap));

    let schema = tool.input_schema();

    // Verify it's an object type
    assert_eq!(schema["type"], "object");

    // Verify it has properties
    assert!(schema["properties"].is_object());
    assert!(schema["properties"]["token_address"].is_object());
    assert!(schema["properties"]["quote_currency"].is_object());

    // Verify required fields
    assert!(schema["required"].is_array());
    let required = schema["required"].as_array().unwrap();
    assert!(required.contains(&json!("token_address")));

    // Verify quote_currency has enum values
    if let Some(quote_currency) = schema["properties"]["quote_currency"].as_object() {
        if let Some(enum_values) = quote_currency.get("enum") {
            assert!(enum_values.is_array());
            let enums = enum_values.as_array().unwrap();
            assert!(enums.contains(&json!("USD")));
            assert!(enums.contains(&json!("ETH")));
        }
    }
}

#[test]
fn test_swap_tokens_tool_name() {
    let wallet_addr: Address = Address::zero();
    let mock_client = MockEthereumClient::new().with_wallet_address(wallet_addr);
    let mock_uniswap = MockUniswapRouter::new();
    let tool = SwapTokensTool::new(Arc::new(mock_client), Arc::new(mock_uniswap));

    assert_eq!(tool.name(), "swap_tokens");
}

#[test]
fn test_swap_tokens_tool_description() {
    let wallet_addr: Address = Address::zero();
    let mock_client = MockEthereumClient::new().with_wallet_address(wallet_addr);
    let mock_uniswap = MockUniswapRouter::new();
    let tool = SwapTokensTool::new(Arc::new(mock_client), Arc::new(mock_uniswap));

    let description = tool.description();
    assert!(!description.is_empty());
    assert!(description.contains("swap") || description.contains("Swap"));
}

#[test]
fn test_swap_tokens_tool_input_schema() {
    let wallet_addr: Address = Address::zero();
    let mock_client = MockEthereumClient::new().with_wallet_address(wallet_addr);
    let mock_uniswap = MockUniswapRouter::new();
    let tool = SwapTokensTool::new(Arc::new(mock_client), Arc::new(mock_uniswap));

    let schema = tool.input_schema();

    // Verify it's an object type
    assert_eq!(schema["type"], "object");

    // Verify it has properties
    assert!(schema["properties"].is_object());
    assert!(schema["properties"]["from_token"].is_object());
    assert!(schema["properties"]["to_token"].is_object());
    assert!(schema["properties"]["amount"].is_object());
    assert!(schema["properties"]["slippage_tolerance"].is_object());

    // Verify required fields
    assert!(schema["required"].is_array());
    let required = schema["required"].as_array().unwrap();
    assert!(required.contains(&json!("from_token")));
    assert!(required.contains(&json!("to_token")));
    assert!(required.contains(&json!("amount")));
}
