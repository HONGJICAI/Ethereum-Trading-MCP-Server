// Mock implementations for testing
use crate::ethereum::client::EthereumClientTrait;
use crate::ethereum::uniswap::SwapSimulation;
use crate::ethereum::uniswap::UniswapRouterTrait;
use anyhow::Result;
use async_trait::async_trait;
use ethers::prelude::*;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Mock Ethereum client for testing
pub struct MockEthereumClient {
    eth_balances: HashMap<Address, Decimal>,
    token_balances: HashMap<(Address, Address), (Decimal, u8)>, // (token, wallet) -> (balance, decimals)
    token_symbols: HashMap<Address, String>,
    wallet_address: Address,
}

impl MockEthereumClient {
    pub fn new() -> Self {
        Self {
            eth_balances: HashMap::new(),
            token_balances: HashMap::new(),
            token_symbols: HashMap::new(),
            wallet_address: Address::zero(),
        }
    }

    pub fn with_wallet_address(mut self, address: Address) -> Self {
        self.wallet_address = address;
        self
    }
    pub fn with_eth_balance(mut self, address: Address, balance: Decimal) -> Self {
        self.eth_balances.insert(address, balance);
        self
    }

    pub fn with_token_balance(
        mut self,
        token: Address,
        wallet: Address,
        balance: Decimal,
        decimals: u8,
    ) -> Self {
        self.token_balances
            .insert((token, wallet), (balance, decimals));
        self
    }

    pub fn with_token_symbol(mut self, token: Address, symbol: String) -> Self {
        self.token_symbols.insert(token, symbol);
        self
    }

    pub async fn get_eth_balance(&self, address: Address) -> Result<Decimal> {
        Ok(self
            .eth_balances
            .get(&address)
            .copied()
            .unwrap_or(Decimal::ZERO))
    }

    pub async fn get_token_balance(
        &self,
        token_address: Address,
        wallet_address: Address,
    ) -> Result<(Decimal, u8)> {
        Ok(self
            .token_balances
            .get(&(token_address, wallet_address))
            .copied()
            .unwrap_or((Decimal::ZERO, 18)))
    }

    pub async fn get_token_symbol(&self, token_address: Address) -> Result<String> {
        Ok(self
            .token_symbols
            .get(&token_address)
            .cloned()
            .unwrap_or_else(|| "UNKNOWN".to_string()))
    }
}

#[async_trait]
impl EthereumClientTrait for MockEthereumClient {
    async fn get_eth_balance(&self, address: Address) -> Result<Decimal> {
        self.get_eth_balance(address).await
    }

    async fn get_token_balance(
        &self,
        token_address: Address,
        wallet_address: Address,
    ) -> Result<(Decimal, u8)> {
        self.get_token_balance(token_address, wallet_address).await
    }

    async fn get_token_symbol(&self, token_address: Address) -> Result<String> {
        self.get_token_symbol(token_address).await
    }

    fn get_wallet_address(&self) -> Address {
        self.wallet_address
    }
}

/// Mock Uniswap router for testing
pub struct MockUniswapRouter {
    prices: HashMap<(Address, Address), Decimal>, // (from_token, to_token) -> price
    swap_simulations: HashMap<(Address, Address), SwapSimulation>,
}

impl MockUniswapRouter {
    pub fn new() -> Self {
        Self {
            prices: HashMap::new(),
            swap_simulations: HashMap::new(),
        }
    }

    pub fn with_price(mut self, from_token: Address, to_token: Address, price: Decimal) -> Self {
        self.prices.insert((from_token, to_token), price);
        self
    }

    pub fn with_swap_simulation(
        mut self,
        from_token: Address,
        to_token: Address,
        simulation: SwapSimulation,
    ) -> Self {
        self.swap_simulations
            .insert((from_token, to_token), simulation);
        self
    }

    pub async fn get_price(
        &self,
        from_token: Address,
        to_token: Address,
        _amount_in: U256,
    ) -> Result<Decimal> {
        self.prices
            .get(&(from_token, to_token))
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Price not found"))
    }

    pub async fn simulate_swap(
        &self,
        from_token: Address,
        to_token: Address,
        _amount_in: U256,
        _wallet_address: Address,
    ) -> Result<SwapSimulation> {
        self.swap_simulations
            .get(&(from_token, to_token))
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Swap simulation not found"))
    }
}

#[async_trait]
impl UniswapRouterTrait for MockUniswapRouter {
    async fn simulate_swap(
        &self,
        from_token: Address,
        to_token: Address,
        amount_in: U256,
        wallet_address: Address,
    ) -> Result<SwapSimulation> {
        self.simulate_swap(from_token, to_token, amount_in, wallet_address)
            .await
    }

    async fn get_price(
        &self,
        from_token: Address,
        to_token: Address,
        amount_in: U256,
    ) -> Result<Decimal> {
        self.get_price(from_token, to_token, amount_in).await
    }
}
