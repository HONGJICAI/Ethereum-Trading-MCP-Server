# Examples

This directory contains examples showing how to use the Ethereum Trading MCP Server as a client.

## Python MCP Client

**File:** `python_mcp_client.py`

A complete example demonstrating how to connect to the Rust MCP server from Python and use all available tools.

### Prerequisites

```bash
pip install -r ../requirements.txt
```

Or install the MCP package directly:

```bash
pip install "mcp[cli]"
```

### Usage

1. Make sure your `.env` file is configured (see main README)

2. Run the example:

```bash
python python_mcp_client.py
```

### What it demonstrates

The example shows how to:

1. **Connect** to the MCP server via stdio
2. **List available tools** from the server
3. **Get ETH balance** for an address
4. **Get token prices** (e.g., UNI token price in USD)
5. **Simulate token swaps** (e.g., WETH → USDC)

### Output Example

```
✅ Connected to Ethereum Trading MCP Server

📋 Available tools:
  - get_balance: Get ETH or ERC20 token balance for an address
  - get_token_price: Get current token price from Uniswap V2
  - swap_tokens: Simulate a token swap on Uniswap V2

🔍 Getting ETH balance...
Result: {"address":"0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045","balance":"...","symbol":"ETH"}

💰 Getting UNI token price in USD...
Result: {"token":"0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984","price":"...","quote_currency":"USD"}

🔄 Simulating token swap (1 WETH -> USDC)...
Result: {"from_token":"0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2","to_token":"...","amount_in":"1.0","amount_out":"..."}
```
