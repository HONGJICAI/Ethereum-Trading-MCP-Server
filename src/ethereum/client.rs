use anyhow::{Context, Result};
use ethers::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;

pub struct EthereumClient {
    provider: Arc<Provider<Http>>,
    wallet: LocalWallet,
    chain_id: u64,
}

impl EthereumClient {
    pub async fn new(rpc_url: &str, private_key: &str, chain_id: u64) -> Result<Self> {
        let provider =
            Provider::<Http>::try_from(rpc_url).context("Failed to connect to Ethereum RPC")?;

        let wallet = private_key
            .parse::<LocalWallet>()
            .context("Failed to parse private key")?
            .with_chain_id(chain_id);

        Ok(Self {
            provider: Arc::new(provider),
            wallet,
            chain_id,
        })
    }

    pub fn get_provider(&self) -> Arc<Provider<Http>> {
        Arc::clone(&self.provider)
    }

    pub fn get_wallet(&self) -> &LocalWallet {
        &self.wallet
    }

    /// Get ETH balance for an address
    pub async fn get_eth_balance(&self, address: Address) -> Result<Decimal> {
        let balance = self
            .provider
            .get_balance(address, None)
            .await
            .context("Failed to get ETH balance")?;

        // Convert from wei to ETH (18 decimals)
        let balance_str = balance.to_string();
        let decimal_balance = Decimal::from_str(&balance_str)?;
        let eth_balance = decimal_balance / Decimal::from(10u64.pow(18));

        Ok(eth_balance)
    }

    /// Get ERC20 token balance for an address
    pub async fn get_token_balance(
        &self,
        token_address: Address,
        wallet_address: Address,
    ) -> Result<(Decimal, u8)> {
        // ERC20 ABI for balanceOf and decimals
        abigen!(
            ERC20,
            r#"[
                function balanceOf(address) external view returns (uint256)
                function decimals() external view returns (uint8)
                function symbol() external view returns (string)
            ]"#
        );

        let contract = ERC20::new(token_address, Arc::clone(&self.provider));

        let balance: U256 = contract
            .balance_of(wallet_address)
            .call()
            .await
            .context("Failed to get token balance")?;

        let decimals: u8 = contract
            .decimals()
            .call()
            .await
            .context("Failed to get token decimals")?;

        let balance_str = balance.to_string();
        let decimal_balance = Decimal::from_str(&balance_str)?;
        let adjusted_balance = decimal_balance / Decimal::from(10u64.pow(decimals as u32));

        Ok((adjusted_balance, decimals))
    }

    /// Get token symbol
    pub async fn get_token_symbol(&self, token_address: Address) -> Result<String> {
        abigen!(
            ERC20,
            r#"[
                function symbol() external view returns (string)
            ]"#
        );

        let contract = ERC20::new(token_address, Arc::clone(&self.provider));
        let symbol = contract
            .symbol()
            .call()
            .await
            .context("Failed to get token symbol")?;

        Ok(symbol)
    }
}
