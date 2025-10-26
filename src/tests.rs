#[cfg(test)]
mod tests {
    use serial_test::serial;

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
}
