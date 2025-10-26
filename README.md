# Ethereum Trading MCP Server

## Overview

This is a Model Context Protocol (MCP) server in Rust that are able to query balances and execute token swaps on Ethereum.

## Features

- **`get_balance`** - Query ETH and ERC20 token balances with proper decimal handling
- **`get_token_price`** - Get current token prices in USD or ETH using Uniswap V2
- **`swap_tokens`** - Simulate token swaps on Uniswap V2 (returns estimates without executing)

## Setup

### Prerequisites

- Rust 1.70+ (`rustup` recommended)
- Python 3.11+ (if test mcp server automatically)
- An Ethereum RPC endpoint (Infura, Alchemy, or public endpoint)
- A private key for transaction signing (for simulation only)

### Installation

1. Clone the repository

2. Copy the example environment file:

```bash
cp .env.example .env
```

3. Edit `.env` with your configuration, or use below testing configuration:

```env
# Using free public endpoint (may be slower than paid services)
ETH_RPC_URL=https://eth.llamarpc.com
PRIVATE_KEY=0000000000000000000000000000000000000000000000000000000000000001
CHAIN_ID=1
```

**⚠️ Security Warning:** Never commit your real private key! The `.env` file is gitignored for safety.

4. Build the project:

```bash
cargo build --release
```

5. Run the server:

```bash
cargo run --release
```

The server reads JSON-RPC requests from stdin and writes responses to stdout.

## Testing

### Unit Tests

Run the unit test suite:

```bash
cargo test
```

### Integration Tests

Run integration tests (requires internet connection to query Ethereum mainnet):

```bash
cargo test --test integration_tests -- --ignored --nocapture
```

These tests verify real blockchain interactions including:
- Querying actual ETH/token balances
- Fetching real Uniswap prices
- Simulating swaps on mainnet

### Coverage Report

Generate a code coverage report:

```bash
cargo tarpaulin --out Html --output-dir coverage --exclude-files "target/*"
```

### Client Examples

See the [`examples/`](./examples/) directory for client implementations that demonstrate how to use the MCP server:

- **Python MCP Client** ([`examples/python_mcp_client.py`](./examples/python_mcp_client.py)) - Full-featured example showing all available tools

To run the Python example:

```bash
cd examples
pip install -r ../requirements.txt
python python_mcp_client.py
```

See [`examples/README.md`](./examples/README.md) for more details.

## Manual Test

You can either test on console app or web UI.

### Console

Check below example MCP tppl calls, then copy & paste request to stdin and check the response from MCP server.

### Web UI

Need to install node.js first.

```bash
npx @modelcontextprotocol/inspector cargo run --release
```

And then click connect, list tools, tool buttons to test.

## Example MCP Tool Calls

You need to minify the json before paste to stdin.

Useful tool to minify: [JSON Monifier](https://codebeautify.org/jsonminifier)

### Initialize

**Request:**

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"example-client","version":"1.0.0"}}}
```

**Response:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "serverInfo": {
      "name": "Ethereum Trading MCP Server",
      "version": "0.1.0"
    },
    "capabilities": {
      "tools": {}
    }
  }
}
```

### Initialized Notification

**IMPORTANT**: After receiving the initialize response, you MUST send this notification:

```json
{"jsonrpc":"2.0","method":"notifications/initialized"}
```

This notification has no response. It signals that the client is ready to begin normal operations.

### List Tools

**Request:**

```json
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
```

**Response:**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "get_balance",
        "description": "Query ETH or ERC20 token balance for a given wallet address",
        "inputSchema": {
          "type": "object",
          "properties": {
            "address": {
              "type": "string",
              "description": "The wallet address to query"
            },
            "token_address": {
              "type": "string",
              "description": "Optional ERC20 token contract address. If omitted, returns ETH balance"
            }
          },
          "required": ["address"]
        }
      }
    ]
  }
}
```

### Call get_balance

Etherscan [url](https://etherscan.io/address/0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045)

**Request:**

```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_balance","arguments":{"address":"0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"}}}
```

**Response:**

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"address\": \"0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045\",\n  \"balance\": \"3.7\",\n  \"symbol\": \"ETH\",\n  \"decimals\": 18\n}"
      }
    ]
  }
}
```

### Call get_token_price

Get the price of UNI token in USD.

**Request:**

```json
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_token_price","arguments":{"token_address":"0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984","quote_currency":"USD"}}}
```

**Response:**

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"token_address\": \"0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984\",\n  \"price\": \"8.50\",\n  \"quote_currency\": \"USD\"\n}"
      }
    ]
  }
}
```

Get the price of UNI token in ETH.

**Request:**

```json
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"get_token_price","arguments":{"token_address":"0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984","quote_currency":"ETH"}}}
```

**Response:**

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"token_address\": \"0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984\",\n  \"price\": \"0.00243\",\n  \"quote_currency\": \"ETH\"\n}"
      }
    ]
  }
}
```

### Call swap_tokens (Simulation)

**Request:**

```json
{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"swap_tokens","arguments":{"from_token":"0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2","to_token":"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48","amount":"1.0","slippage_tolerance":0.5}}}
```

**Response:**

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"from_token\": \"0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2\",\n  \"to_token\": \"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48\",\n  \"amount_in\": \"1.0\",\n  \"estimated_amount_out\": \"3500.0\",\n  \"minimum_amount_out\": \"3482.5\",\n  \"gas_estimate\": \"150000\",\n  \"gas_price_gwei\": \"30\",\n  \"estimated_gas_cost_eth\": \"0.0045\"\n}"
      }
    ]
  }
}
```

## Design Decisions

**Architecture:** The system is built with a modular architecture separating concerns into distinct modules: Ethereum client operations, MCP server protocol handling, and individual tool implementations. This makes the codebase maintainable and testable.

**Ethereum Library:** We use `ethers-rs` for Ethereum interactions due to its mature ecosystem, strong type safety, and comprehensive ABI encoding/decoding capabilities. The library provides excellent support for contract interactions and RPC operations.

**Uniswap Integration:** The implementation uses Uniswap V2 Router for price queries and swap simulations. Swaps are simulated using `getAmountsOut` for price estimation and gas estimation without executing transactions on-chain. This provides safe, read-only operations.

**Financial Precision:** All financial calculations use `rust_decimal` to avoid floating-point precision issues. Token amounts are properly converted between human-readable decimals and blockchain wei/token units.

**MCP Protocol:** The server implements the JSON-RPC 2.0 specification manually for the MCP protocol, communicating via stdin/stdout. This provides maximum compatibility with MCP clients and tools.

## Known Limitations & Assumptions

1. **Decimals Assumption:** The swap simulation assumes 18 decimals for input amounts. Production code should query each token's decimals() function.

2. **Uniswap V2 Only:** Currently only supports Uniswap V2. V3 has different mechanics (concentrated liquidity) that would require separate implementation.

3. **Mainnet Focus:** Configuration is optimized for Ethereum mainnet. Other networks (L2s, testnets) would need different contract addresses.

4. **No Transaction Execution:** The `swap_tokens` tool only simulates swaps and estimates gas. It does not execute real transactions, providing safety for exploratory use.

5. **Price Oracle:** Token prices use Uniswap V2 liquidity pools. For low-liquidity tokens, prices may not be accurate. Production systems should aggregate multiple price sources.

6. **Error Handling:** While comprehensive, some edge cases (network failures, invalid tokens) may not have perfect user-facing error messages.

7. **Rate Limiting:** No built-in rate limiting for RPC calls. Heavy usage may hit provider limits.
