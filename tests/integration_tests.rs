use ethereum_trading_mcp_server::*;
use ethers::signers::Signer;
use tokio;

// Integration tests that query real Ethereum data
// These tests require an internet connection and working RPC endpoint

#[tokio::test]
#[ignore] // Use `cargo test -- --ignored` to run these tests
async fn test_get_balance_real_eth() {
    // This test queries Vitalik's real ETH balance
    let config = config::Config {
        eth_rpc_url: "https://eth.llamarpc.com".to_string(),
        private_key: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
        chain_id: 1,
    };

    let client =
        ethereum::EthereumClient::new(&config.eth_rpc_url, &config.private_key, config.chain_id)
            .await
            .expect("Failed to create Ethereum client");

    // Vitalik's address
    let vitalik_address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
        .parse()
        .expect("Invalid address");

    let balance = client
        .get_eth_balance(vitalik_address)
        .await
        .expect("Failed to get balance");

    // Vitalik should have some ETH (at least 0.01 ETH, probably much more)
    assert!(balance > rust_decimal::Decimal::new(1, 2)); // > 0.01 ETH
    println!("✓ Vitalik's ETH balance: {} ETH", balance);
}

#[tokio::test]
#[ignore]
async fn test_get_balance_real_usdc() {
    // This test queries a real USDC balance
    let config = config::Config {
        eth_rpc_url: "https://eth.llamarpc.com".to_string(),
        private_key: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
        chain_id: 1,
    };

    let client =
        ethereum::EthereumClient::new(&config.eth_rpc_url, &config.private_key, config.chain_id)
            .await
            .expect("Failed to create Ethereum client");

    // Binance hot wallet
    let binance_address = "0x28C6c06298d514Db089934071355E5743bf21d60"
        .parse()
        .expect("Invalid address");

    // USDC contract
    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        .parse()
        .expect("Invalid USDC address");

    let (balance, decimals) = client
        .get_token_balance(usdc_address, binance_address)
        .await
        .expect("Failed to get USDC balance");

    assert_eq!(decimals, 6); // USDC has 6 decimals
    assert!(balance > rust_decimal::Decimal::ZERO); // Binance should have USDC
    println!(
        "✓ Binance USDC balance: {} USDC (decimals: {})",
        balance, decimals
    );
}

#[tokio::test]
#[ignore]
async fn test_get_token_symbol_real() {
    let config = config::Config {
        eth_rpc_url: "https://eth.llamarpc.com".to_string(),
        private_key: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
        chain_id: 1,
    };

    let client =
        ethereum::EthereumClient::new(&config.eth_rpc_url, &config.private_key, config.chain_id)
            .await
            .expect("Failed to create Ethereum client");

    // USDC contract
    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        .parse()
        .expect("Invalid USDC address");

    let symbol = client
        .get_token_symbol(usdc_address)
        .await
        .expect("Failed to get token symbol");

    assert_eq!(symbol, "USDC");
    println!("✓ Token symbol: {}", symbol);
}

#[tokio::test]
#[ignore]
async fn test_uniswap_price_real() {
    let config = config::Config {
        eth_rpc_url: "https://eth.llamarpc.com".to_string(),
        private_key: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
        chain_id: 1,
    };

    let client =
        ethereum::EthereumClient::new(&config.eth_rpc_url, &config.private_key, config.chain_id)
            .await
            .expect("Failed to create Ethereum client");

    let uniswap = ethereum::UniswapV2Router::new(client.get_provider());

    // Get WETH price in USDC (should be around current ETH price)
    let weth_address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
        .parse()
        .expect("Invalid WETH address");

    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        .parse()
        .expect("Invalid USDC address");

    // Query price for 1 WETH (18 decimals)
    let one_weth = ethers::types::U256::from(10u64.pow(18));

    let price = uniswap
        .get_price(weth_address, usdc_address, one_weth)
        .await
        .expect("Failed to get price");

    // ETH price should be between $100 and $100,000 (reasonable bounds)
    // Adjust for USDC having 6 decimals
    let adjusted_price = price * rust_decimal::Decimal::from(10u64.pow(12));

    assert!(adjusted_price > rust_decimal::Decimal::from(100));
    assert!(adjusted_price < rust_decimal::Decimal::from(100000));
    println!("✓ 1 WETH = {} USDC", adjusted_price);
}

#[tokio::test]
#[ignore]
async fn test_uniswap_swap_simulation_real() {
    let config = config::Config {
        eth_rpc_url: "https://eth.llamarpc.com".to_string(),
        private_key: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
        chain_id: 1,
    };

    let client = std::sync::Arc::new(
        ethereum::EthereumClient::new(&config.eth_rpc_url, &config.private_key, config.chain_id)
            .await
            .expect("Failed to create Ethereum client"),
    );

    let uniswap = ethereum::UniswapV2Router::new(client.get_provider());

    // Simulate swapping 1 WETH to USDC
    let weth_address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
        .parse()
        .expect("Invalid WETH address");

    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        .parse()
        .expect("Invalid USDC address");

    let one_weth = ethers::types::U256::from(10u64.pow(18));
    let wallet_address = client.get_wallet().address();

    let simulation = uniswap
        .simulate_swap(weth_address, usdc_address, one_weth, wallet_address)
        .await
        .expect("Failed to simulate swap");

    // Should get some USDC out (at least $100 worth)
    let min_usdc = ethers::types::U256::from(100 * 10u64.pow(6)); // 100 USDC
    assert!(simulation.amount_out > min_usdc);

    // Gas estimate should be reasonable (between 100k and 500k gas)
    assert!(simulation.gas_estimate > ethers::types::U256::from(100000));
    assert!(simulation.gas_estimate < ethers::types::U256::from(500000));

    println!("✓ Swap simulation:");
    println!("  Input: 1 WETH");
    println!(
        "  Output: {} USDC",
        simulation.amount_out.as_u128() as f64 / 1e6
    );
    println!("  Gas estimate: {}", simulation.gas_estimate);
    println!(
        "  Gas price: {} gwei",
        simulation.gas_price.as_u128() as f64 / 1e9
    );
}

#[tokio::test]
#[ignore]
async fn test_mcp_get_balance_tool_real() {
    use serde_json::json;
    use tools::Tool;

    let config = config::Config {
        eth_rpc_url: "https://eth.llamarpc.com".to_string(),
        private_key: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
        chain_id: 1,
    };

    let client = std::sync::Arc::new(
        ethereum::EthereumClient::new(&config.eth_rpc_url, &config.private_key, config.chain_id)
            .await
            .expect("Failed to create Ethereum client"),
    );

    let tool = tools::GetBalanceTool::new(client);

    // Test with Vitalik's address
    let params = json!({
        "address": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
    });

    let result = tool.execute(params).await.expect("Tool execution failed");

    // Verify result structure
    assert!(result.is_object());
    assert!(result.get("address").is_some());
    assert!(result.get("balance").is_some());
    assert!(result.get("symbol").is_some());
    assert_eq!(result.get("symbol").unwrap().as_str().unwrap(), "ETH");

    println!(
        "✓ MCP Tool Result: {}",
        serde_json::to_string_pretty(&result).unwrap()
    );
}
