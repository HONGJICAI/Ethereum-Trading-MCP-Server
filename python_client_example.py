"""
Python MCP Client for Ethereum Trading MCP Server

This demonstrates how to connect to the Rust MCP server via stdio.
Install: pip install "mcp[cli]"
"""

import asyncio
import os
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client


async def main():
    # Configure the Rust MCP server
    server_params = StdioServerParameters(
        command="cargo",
        args=["run", "--release"],
        env={
            "ETH_RPC_URL": os.getenv("ETH_RPC_URL", "https://eth.llamarpc.com"),
            "PRIVATE_KEY": os.getenv("PRIVATE_KEY", "0000000000000000000000000000000000000000000000000000000000000001"),
            "CHAIN_ID": "1"
        }
    )

    async with stdio_client(server_params) as (read, write):
        async with ClientSession(read, write) as session:
            # Initialize the connection
            await session.initialize()
            print("✅ Connected to Ethereum Trading MCP Server\n")

            # List available tools
            tools = await session.list_tools()
            print("📋 Available tools:")
            for tool in tools.tools:
                print(f"  - {tool.name}: {tool.description}")
            print()

            # Example 1: Get ETH balance
            print("🔍 Getting ETH balance...")
            result = await session.call_tool(
                "get_balance",
                arguments={
                    "address": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
                }
            )
            print(f"Result: {result.content[0].text}\n")

            # Example 2: Get token price (UNI in USD)
            print("💰 Getting UNI token price in USD...")
            result = await session.call_tool(
                "get_token_price",
                arguments={
                    "token_address": "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984",
                    "quote_currency": "USD"
                }
            )
            print(f"Result: {result.content[0].text}\n")

            # Example 3: Simulate swap (WETH -> USDC)
            print("🔄 Simulating token swap (1 WETH -> USDC)...")
            result = await session.call_tool(
                "swap_tokens",
                arguments={
                    "from_token": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
                    "to_token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                    "amount": "1.0",
                    "slippage_tolerance": 0.5
                }
            )
            print(f"Result: {result.content[0].text}\n")


if __name__ == "__main__":
    asyncio.run(main())
