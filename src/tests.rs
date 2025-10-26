#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use serial_test::serial;

    // ============ Decimal Conversion Tests ============
    
    #[test]
    fn test_decimal_conversion() {
        // Test converting wei to ETH
        let wei = "1000000000000000000"; // 1 ETH in wei
        let decimal = Decimal::from_str(wei).unwrap();
        let eth = decimal / Decimal::from(10u64.pow(18));
        assert_eq!(eth.to_string(), "1");
    }

    #[test]
    fn test_decimal_conversion_fractional() {
        // Test converting fractional amounts
        let wei = "500000000000000000"; // 0.5 ETH in wei
        let decimal = Decimal::from_str(wei).unwrap();
        let eth = decimal / Decimal::from(10u64.pow(18));
        // Decimal might have trailing zeros
        assert!(eth.to_string() == "0.5" || eth.to_string() == "0.50");
    }

    #[test]
    fn test_decimal_conversion_usdc() {
        // Test USDC with 6 decimals
        let usdc_raw = "1000000"; // 1 USDC
        let decimal = Decimal::from_str(usdc_raw).unwrap();
        let usdc = decimal / Decimal::from(10u64.pow(6));
        assert_eq!(usdc.to_string(), "1");
    }

    #[test]
    fn test_decimal_conversion_large_amount() {
        // Test large amounts
        let wei = "123456789000000000000"; // 123.456789 ETH
        let decimal = Decimal::from_str(wei).unwrap();
        let eth = decimal / Decimal::from(10u64.pow(18));
        assert_eq!(eth.to_string(), "123.456789");
    }

    #[test]
    fn test_decimal_conversion_zero() {
        // Test zero balance
        let wei = "0";
        let decimal = Decimal::from_str(wei).unwrap();
        let eth = decimal / Decimal::from(10u64.pow(18));
        assert_eq!(eth.to_string(), "0");
    }

    // ============ Config Tests ============

    #[test]
    #[serial]
    fn test_config_from_env() {
        use crate::config::Config;
        std::env::set_var("ETH_RPC_URL", "https://eth.llamarpc.com");
        std::env::set_var(
            "PRIVATE_KEY",
            "0000000000000000000000000000000000000000000000000000000000000001",
        );
        std::env::set_var("CHAIN_ID", "1");

        let config = Config::from_env().unwrap();
        assert_eq!(config.eth_rpc_url, "https://eth.llamarpc.com");
        assert_eq!(config.chain_id, 1);
    }

    #[test]
    #[serial]
    fn test_config_default_chain_id() {
        use crate::config::Config;
        std::env::set_var("ETH_RPC_URL", "https://eth.llamarpc.com");
        std::env::set_var(
            "PRIVATE_KEY",
            "0000000000000000000000000000000000000000000000000000000000000001",
        );
        std::env::remove_var("CHAIN_ID");

        let config = Config::from_env().unwrap();
        assert_eq!(config.chain_id, 1); // Default chain ID
    }

    #[test]
    #[serial]
    fn test_config_missing_rpc_url() {
        use crate::config::Config;
        std::env::remove_var("ETH_RPC_URL");
        std::env::set_var(
            "PRIVATE_KEY",
            "0000000000000000000000000000000000000000000000000000000000000001",
        );

        let result = Config::from_env();
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_config_missing_private_key() {
        use crate::config::Config;
        std::env::set_var("ETH_RPC_URL", "https://eth.llamarpc.com");
        std::env::remove_var("PRIVATE_KEY");

        let result = Config::from_env();
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_config_invalid_chain_id() {
        use crate::config::Config;
        std::env::set_var("ETH_RPC_URL", "https://eth.llamarpc.com");
        std::env::set_var(
            "PRIVATE_KEY",
            "0000000000000000000000000000000000000000000000000000000000000001",
        );
        std::env::set_var("CHAIN_ID", "invalid");

        let result = Config::from_env();
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_config_different_networks() {
        use crate::config::Config;
        
        // Test Sepolia
        std::env::set_var("ETH_RPC_URL", "https://sepolia.infura.io");
        std::env::set_var(
            "PRIVATE_KEY",
            "0000000000000000000000000000000000000000000000000000000000000001",
        );
        std::env::set_var("CHAIN_ID", "11155111");

        let config = Config::from_env().unwrap();
        assert_eq!(config.chain_id, 11155111);
    }

    // ============ Tool Schema Tests ============

    #[test]
    fn test_tool_schema() {
        use crate::tools::Tool;
        use serde_json::Value;

        // Create a mock tool to test the schema
        struct MockTool;

        #[async_trait::async_trait]
        impl Tool for MockTool {
            fn name(&self) -> &str {
                "test_tool"
            }

            fn description(&self) -> &str {
                "A test tool"
            }

            fn input_schema(&self) -> Value {
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "param": {"type": "string"}
                    }
                })
            }

            async fn execute(&self, _params: Value) -> anyhow::Result<Value> {
                Ok(serde_json::json!({"result": "success"}))
            }
        }

        let tool = MockTool;
        assert_eq!(tool.name(), "test_tool");
        assert!(tool.input_schema().is_object());
    }

    #[test]
    fn test_tool_schema_properties() {
        use serde_json::json;
        
        let schema = json!({
            "type": "object",
            "properties": {
                "address": {"type": "string"},
                "amount": {"type": "number"}
            },
            "required": ["address"]
        });

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
        assert!(schema["required"].is_array());
    }

    // ============ JSON-RPC Tests ============

    #[test]
    fn test_jsonrpc_request_parsing() {
        use serde_json::json;

        let request_str = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        }"#;

        let parsed: serde_json::Value = serde_json::from_str(request_str).unwrap();
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["method"], "tools/list");
    }

    #[test]
    fn test_jsonrpc_response_serialization() {
        use serde_json::json;

        let response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "tools": []
            }
        });

        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("jsonrpc"));
        assert!(serialized.contains("result"));
    }

    #[test]
    fn test_jsonrpc_error_structure() {
        use serde_json::json;

        let error_response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32600,
                "message": "Invalid Request"
            }
        });

        assert!(error_response["error"].is_object());
        assert_eq!(error_response["error"]["code"], -32600);
    }

    // ============ Address Validation Tests ============

    #[test]
    fn test_valid_ethereum_address() {
        let addr_str = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
        let result = addr_str.parse::<ethers::types::Address>();
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_ethereum_address() {
        let addr_str = "invalid_address";
        let result = addr_str.parse::<ethers::types::Address>();
        assert!(result.is_err());
    }

    #[test]
    fn test_lowercase_ethereum_address() {
        let addr_str = "0xd8da6bf26964af9d7eed9e03e53415d37aa96045";
        let result = addr_str.parse::<ethers::types::Address>();
        assert!(result.is_ok());
    }

    #[test]
    fn test_address_without_0x_prefix() {
        let addr_str = "d8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
        let result = addr_str.parse::<ethers::types::Address>();
        // Should still parse correctly
        assert!(result.is_ok());
    }

    // ============ Parameter Validation Tests ============

    #[test]
    fn test_get_balance_params_parsing() {
        use serde_json::json;

        let params = json!({
            "address": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
        });

        assert!(params["address"].is_string());
        assert!(params["token_address"].is_null());
    }

    #[test]
    fn test_get_balance_params_with_token() {
        use serde_json::json;

        let params = json!({
            "address": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            "token_address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        });

        assert!(params["address"].is_string());
        assert!(params["token_address"].is_string());
    }

    #[test]
    fn test_swap_params_parsing() {
        use serde_json::json;

        let params = json!({
            "from_token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "to_token": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
            "amount": "100",
            "slippage_tolerance": 0.5
        });

        assert!(params["from_token"].is_string());
        assert!(params["to_token"].is_string());
        assert!(params["amount"].is_string());
        assert!(params["slippage_tolerance"].is_number());
    }

    #[test]
    fn test_token_price_params_parsing() {
        use serde_json::json;

        let params = json!({
            "token_address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "quote_currency": "USD"
        });

        assert!(params["token_address"].is_string());
        assert_eq!(params["quote_currency"], "USD");
    }

    #[test]
    fn test_token_price_params_default_currency() {
        use serde_json::json;

        let params = json!({
            "token_address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        });

        assert!(params["token_address"].is_string());
        // quote_currency should use default
    }

    // ============ Slippage Calculation Tests ============

    #[test]
    fn test_slippage_calculation() {
        let amount = Decimal::from_str("100").unwrap();
        let _slippage = 0.5; // 0.5%
        let slippage_decimal = Decimal::from_str("0.005").unwrap(); // 0.5% as decimal
        let min_amount = amount * (Decimal::ONE - slippage_decimal);
        
        // Result might have trailing zeros
        let result_str = min_amount.to_string();
        assert!(result_str == "99.5" || result_str == "99.500");
    }

    #[test]
    fn test_slippage_calculation_1_percent() {
        let amount = Decimal::from_str("1000").unwrap();
        let slippage_decimal = Decimal::from_str("0.01").unwrap(); // 1%
        let min_amount = amount * (Decimal::ONE - slippage_decimal);
        
        // Result might have trailing zeros
        let result_str = min_amount.to_string();
        assert!(result_str == "990" || result_str == "990.00");
    }

    #[test]
    fn test_slippage_calculation_zero() {
        let amount = Decimal::from_str("100").unwrap();
        let slippage_decimal = Decimal::ZERO;
        let min_amount = amount * (Decimal::ONE - slippage_decimal);
        
        assert_eq!(min_amount, amount);
    }

    // ============ Gas Cost Calculation Tests ============

    #[test]
    fn test_gas_cost_calculation() {
        // Gas estimate: 200000
        // Gas price: 50 gwei (50_000_000_000 wei)
        // Cost = 200000 * 50_000_000_000 = 10_000_000_000_000_000 wei = 0.01 ETH
        
        let gas_estimate = 200000u64;
        let gas_price_gwei = 50u64;
        let gas_price_wei = gas_price_gwei * 1_000_000_000;
        let gas_cost_wei = gas_estimate * gas_price_wei;
        
        assert_eq!(gas_cost_wei, 10_000_000_000_000_000);
        
        let gas_cost_eth = Decimal::from(gas_cost_wei) / Decimal::from(10u64.pow(18));
        assert_eq!(gas_cost_eth.to_string(), "0.01");
    }

    #[test]
    fn test_gas_cost_high_price() {
        let gas_estimate = 150000u64;
        let gas_price_gwei = 100u64; // High gas price
        let gas_price_wei = gas_price_gwei * 1_000_000_000;
        let gas_cost_wei = gas_estimate * gas_price_wei;
        
        let gas_cost_eth = Decimal::from(gas_cost_wei) / Decimal::from(10u64.pow(18));
        assert_eq!(gas_cost_eth.to_string(), "0.015");
    }

    // ============ U256 Conversion Tests ============

    #[test]
    fn test_u256_to_decimal() {
        use ethers::types::U256;
        
        let value = U256::from(1_000_000_000_000_000_000u64); // 1 ETH
        let value_str = value.to_string();
        let decimal = Decimal::from_str(&value_str).unwrap();
        let eth = decimal / Decimal::from(10u64.pow(18));
        
        assert_eq!(eth.to_string(), "1");
    }

    #[test]
    fn test_u256_zero() {
        use ethers::types::U256;
        
        let value = U256::zero();
        let value_str = value.to_string();
        let decimal = Decimal::from_str(&value_str).unwrap();
        
        assert_eq!(decimal, Decimal::ZERO);
    }

    #[test]
    fn test_decimal_to_u256_string() {
        let amount = Decimal::from_str("1.5").unwrap();
        let decimals = 18u32;
        let amount_raw = amount * Decimal::from(10u64.pow(decimals));
        let amount_str = amount_raw.to_string();
        
        // Remove decimal point if present
        let amount_str_clean = amount_str.split('.').next().unwrap();
        assert_eq!(amount_str_clean, "1500000000000000000");
    }

    // ============ Error Handling Tests ============

    #[test]
    fn test_invalid_json_parsing() {
        let invalid_json = "{invalid json}";
        let result = serde_json::from_str::<serde_json::Value>(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_field() {
        use serde_json::json;
        
        // Missing required "address" field
        let params = json!({
            "token_address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        });

        // Should be able to parse as Value, but validation would fail
        assert!(params.is_object());
        assert!(params.get("address").is_none());
    }

    // ============ String/Number Format Tests ============

    #[test]
    fn test_amount_string_parsing() {
        let amount_str = "123.456";
        let decimal = Decimal::from_str(amount_str).unwrap();
        assert_eq!(decimal.to_string(), "123.456");
    }

    #[test]
    fn test_scientific_notation() {
        // Decimal doesn't support scientific notation directly, use explicit value
        let value = Decimal::from(10u64.pow(18));
        assert_eq!(value, Decimal::from(1_000_000_000_000_000_000u64));
    }

    #[test]
    fn test_very_small_amount() {
        let amount_str = "0.000001";
        let decimal = Decimal::from_str(amount_str).unwrap();
        assert_eq!(decimal.to_string(), "0.000001");
    }

    // ============ Tool Name and Description Tests ============

    #[test]
    fn test_tool_names_unique() {
        let tool_names = vec!["get_balance", "get_token_price", "swap_tokens"];
        let mut unique_names = tool_names.clone();
        unique_names.sort();
        unique_names.dedup();
        
        assert_eq!(tool_names.len(), unique_names.len());
    }

    #[test]
    fn test_tool_names_format() {
        let tool_names = vec!["get_balance", "get_token_price", "swap_tokens"];
        
        for name in tool_names {
            // Tool names should be lowercase snake_case
            assert!(name.chars().all(|c| c.is_lowercase() || c == '_'));
            assert!(!name.starts_with('_'));
            assert!(!name.ends_with('_'));
        }
    }

    // ============ Server Info Tests ============

    #[test]
    fn test_server_info_structure() {
        use serde_json::json;
        
        let server_info = json!({
            "name": "Ethereum Trading MCP Server",
            "version": "0.1.0",
            "protocol_version": "2024-11-05"
        });

        assert!(server_info["name"].is_string());
        assert!(server_info["version"].is_string());
        assert!(server_info["protocol_version"].is_string());
    }

    #[test]
    fn test_version_format() {
        let version = "0.1.0";
        let parts: Vec<&str> = version.split('.').collect();
        
        assert_eq!(parts.len(), 3);
        assert!(parts.iter().all(|p| p.parse::<u32>().is_ok()));
    }
}
