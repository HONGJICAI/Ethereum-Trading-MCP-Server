#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_decimal_conversion() {
        // Test converting wei to ETH
        let wei = "1000000000000000000"; // 1 ETH in wei
        let decimal = Decimal::from_str(wei).unwrap();
        let eth = decimal / Decimal::from(10u64.pow(18));
        assert_eq!(eth.to_string(), "1");
    }

    #[test]
    fn test_config_from_env() {
        use crate::config::Config;
        std::env::set_var("ETH_RPC_URL", "https://eth.llamarpc.com");
        std::env::set_var("PRIVATE_KEY", "0000000000000000000000000000000000000000000000000000000000000001");
        std::env::set_var("CHAIN_ID", "1");

        let config = Config::from_env().unwrap();
        assert_eq!(config.eth_rpc_url, "https://eth.llamarpc.com");
        assert_eq!(config.chain_id, 1);
    }

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
