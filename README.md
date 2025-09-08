# ETH Partner — Ethereum Ai-Assistant

This repository is an experiment in Spec‑Driven, AI‑assisted development for
Ethereum workflows. The goal is to explore how an AI assistant can help you
reason about, test, and execute on‑chain actions end‑to‑end, from natural
language prompts to concrete JSON‑RPC calls and signed transactions.

At a glance:
- Proof‑of‑Concept (POC), not production software.
- Focused on Ethereum developer ergonomics and rapid experimentation.
- Consists of two Rust crates in a Cargo workspace:
  - `repl/` — an interactive AI assistant REPL that can use tools.
  - `mcp-wallet/` — a minimal MCP‑style wallet server for accounts and txs.


## Talk to the AI assistant

The assistant is designed to handle Ethereum‑centric prompts. Example session:

```
> Alice is 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
> Bob is 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
> send 1 ETH from Alice to Bob
> How much USDC does Alice have?
> Is Uniswap V2 Router (0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D) deployed?
> Use Uniswap V2 Router to swap 10 ETH for USDC on Alice's account.
> How do I calculate slippage for Uniswap V3?
> What's the difference between exactInput and exactOutput?
> Show me the SwapRouter contract interface
```

Depending on configuration, the agent will:
- Resolve aliases like “Alice”/“Bob” to addresses.
- Query balances and contract code via Ethereum JSON‑RPC.
- Create, sign, and (optionally) send EIP‑1559 transactions via the wallet.
- Use web search (optional) to retrieve protocol documentation (e.g., Uniswap).


## Quick start

Prerequisites:
- Rust toolchain (stable). Install from https://www.rust-lang.org/
- Optional for local chain: Foundry `anvil` (https://book.getfoundry.sh)

1) Clone and enter the repo, then (optionally) prepare a `.env`:

```
cp .env.example .env
# edit values as needed
```

Common variables:
- `ETH_RPC_URL` (default: `http://127.0.0.1:8545`)
- `CHAIN_ID` (e.g., `31337` for Anvil)
- `WALLET_FILE` (path to a JSON wallet file for the POC wallet)
- Optional Google CSE keys to enable web search in the REPL:
  - `GOOGLE_SEARCH_API_KEY`
  - `GOOGLE_SEARCH_ENGINE_ID`

2) (Optional) Start a local Ethereum node with Anvil:

```
anvil
```

3) Run the REPL assistant:

```
cargo run -p repl
```

Inside the REPL you can type natural language prompts like in the examples
above. The agent will register available tools (wallet, web search, etc.) based
on your environment.


## Workspace layout

- `Cargo.toml` — workspace definition for all crates
- `repl/` — interactive assistant that wires LLM + tools
  - see `repl/README.md` for configuration and usage details
- `mcp-wallet/` — minimal wallet server exposing MCP‑style tools
  - see `mcp-wallet/README.md` for wallet operations and format
- `tasks/` — product requirement docs and task lists used to guide the build


## Architecture

High-level view of how the components interact:

```
┌───────────────────────────┐        Ethereum ops           ┌──────────────────────┐
│          repl/            │ ────────────────────────────▶ │     mcp-wallet/      │
│  AI Assistant (LLM + UX)  │                               │  MCP-style wallet    │
│  • natural language input │ ◀──────────────────────────── │  server (stdio IPC)  │
│  • tool orchestration     │                               └─────────┬────────────┘
└─────────────┬─────────────┘                                         │
              │                                                       │ JSON-RPC
              │                                                       ▼
              │                                         ┌─────────────────────────┐
              │                                         │   Ethereum JSON-RPC     │
              │                                         │  (Anvil, geth, infura)  │
              │                                         └─────────────────────────┘
              │
              │ optional
              ▼
   ┌─────────────────────────┐
   │    Web Search Tool      │
   │  (Google CSE JSON API)  │
   └─────────────────────────┘
```

Key points:
- The `repl/` crate hosts the interactive agent and routes intents to tools.
- The `mcp-wallet/` crate runs as an embedded or external process and exposes
  wallet/account/transaction tools over a simple MCP-style stdio protocol.
- Wallet tools communicate with an Ethereum node via JSON-RPC to read state and
  broadcast signed EIP‑1559 transactions.
- An optional web search tool helps the agent retrieve protocol docs/snippets
  (e.g., Uniswap v2/v3) to answer conceptual questions.

Typical flow (example: "send 1 ETH from Alice to Bob"):
1. `repl/` parses intent, resolves `Alice`/`Bob` to addresses/aliases.
2. Calls `mcp-wallet` tool to create and sign an EIP‑1559 transaction.
3. `mcp-wallet` submits the raw tx via JSON‑RPC and returns the hash.
4. `repl/` optionally polls for a receipt and summarizes the result.


## Common tasks cheat sheet

Quick examples you can paste into the REPL. Adjust addresses and values as
needed.

### Define simple aliases

```
Alice is 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
Bob is 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
```

### Check ETH balance

```
What is Alice's ETH balance?
```

Or by address:

```
What is the ETH balance of 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266?
```

### Transfer ETH

```
Send 1 ETH from Alice to Bob
```

The assistant will create/sign an EIP‑1559 transaction via the wallet and
submit it through JSON‑RPC, returning a transaction hash.

### Get transaction status

```
Show the receipt for 0x<your_tx_hash>
```

The agent will fetch the receipt and summarize status, gas used, and logs.

### Check if a contract is deployed at an address

```
Is there code deployed at 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D?
```

On success, the agent confirms non‑empty bytecode.

### Outline: swap ETH->USDC via Uniswap V2

This POC focuses on wallet primitives. A full swap helper may not be available
yet, but the agent can help outline steps and produce calldata:

```
Use Uniswap V2 Router (0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D) to swap
10 ETH for USDC on Alice's account. Assume a reasonable slippage buffer and
show me the function signature and parameters you will use.
```

The agent can:
- Identify `swapExactETHForTokens` inputs, path [WETH, USDC], recipient, deadline.
- Propose `amountOutMin` based on quotes and slippage.
- Construct calldata and estimate gas; you review before sending.

Note: For Uniswap V3 topics (slippage, `exactInput` vs `exactOutput`), see the
next addition if enabled.


## Uniswap V2/V3 mini‑guide

This POC can help you reason about swaps and construct calldata, but it does not
ship a full DEX router integration. Use these notes to guide prompts and review
the generated parameters before sending transactions.

### Slippage basics

- Slippage protects you from price movement or insufficient liquidity between
  quote and execution.
- For V2 `swapExactETHForTokens`, set `amountOutMin = quoteOut * (1 - slippage)`
  where slippage is a decimal (e.g., 0.01 for 1%).
- For V3 `exactInput`, set `amountOutMinimum` similarly. For `exactOutput`, set
  `amountInMaximum = quoteIn * (1 + slippage)`.

### exactInput vs exactOutput (Uniswap V3)

- `exactInput`:
  - You specify exact input amount, you receive at least `amountOutMinimum`.
  - Good when you want to spend a fixed amount, accept variable output.
- `exactOutput`:
  - You specify exact desired output, spend up to `amountInMaximum`.
  - Good when you must receive a fixed quantity, accept variable cost.

### Common router addresses (Ethereum mainnet)

- Uniswap V2 Router: `0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D`
- Uniswap V3 SwapRouter: `0xE592427A0AEce92De3Edee1F18E0157C05861564`

Always confirm the correct address for your network (testnets differ).

### Helpful prompts

```
Explain how to pick amountOutMin for Uniswap V2 given a 1% slippage on a
10 ETH -> USDC swap. Show the formula and a numeric example.
```

```
What is the difference between exactInput and exactOutput on Uniswap V3?
When should I use each? Provide example parameters.
```

```
Construct calldata for Uniswap V3 exactInput from WETH to USDC for 5 ETH on
Alice's account, using a 0.3% fee pool and 1% slippage. Show the path encoding
and amountOutMinimum calculation. Do not send, just show the data.
```

References:
- Uniswap V2 docs: https://docs.uniswap.org/contracts/v2/overview
- Uniswap V2 Router (Etherscan, ABI): https://etherscan.io/address/0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D#code
- Uniswap V3 docs (core + periphery): https://docs.uniswap.org/contracts/v3/overview
- Uniswap V3 SwapRouter (Etherscan, ABI): https://etherscan.io/address/0xE592427A0AEce92De3Edee1F18E0157C05861564#code


## What this POC does today

- Registers a set of Ethereum tools in the REPL, including:
  - Account management (`new_account`, `list_accounts`, aliases)
  - Balance queries (`eth_get_balance`)
  - Transaction creation/signing/sending (EIP‑1559)
  - Convenience ETH transfers (`eth_transfer_eth`)
  - Basic tx/receipt lookups
- Works against any Ethereum JSON‑RPC endpoint (local Anvil or remote)
- Can optionally use Google Programmable Search to fetch docs and examples for
  protocols like Uniswap, aiding Q&A about `exactInput` vs `exactOutput`,
  slippage, router contract ABIs, etc.


## Limitations & safety

- This is a research/POC codebase. Do not use with real funds.
- Keys may be stored unencrypted in a local JSON file for convenience.
- The agent may make mistakes; always verify suggested transactions and
  contracts. Review raw data (to, value, calldata, gas) before sending.


## Development

- Run tests: `cargo test`
- Lint/format: `cargo fmt --all` and `cargo clippy`
- REPL details and troubleshooting: see `repl/README.md`
- Wallet server details: see `mcp-wallet/README.md`


## License

This project is provided for research and educational purposes. See repository
files for license information, if provided. If absent, treat as “all rights
reserved” by the authors until a license is added.
