#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use serial_test::serial;
    use std::str::FromStr;

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
}
