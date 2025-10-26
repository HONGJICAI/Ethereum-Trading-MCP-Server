pub mod client;
pub mod mock;
pub mod uniswap;

pub use client::{EthereumClient, EthereumClientTrait};

#[cfg(test)]
pub use mock::{MockEthereumClient, MockUniswapRouter};
pub use uniswap::{SwapSimulation, UniswapRouterTrait, UniswapV2Router};
