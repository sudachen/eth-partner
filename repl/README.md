# REPL AI Assistant

This crate provides a simple REPL (Read-Eval-Print Loop) AI assistant that uses the `rig` framework to interact with large language models and external tools.

## Features

- Interactive REPL interface.
- Extensible toolset for interacting with external services.
- Configuration managed through a simple JSON file.
- Powered by Google's Gemini Pro via the `rig` framework.
- Optional web search tool backed by Google Programmable Search Engine (CSE).

## Configuration

A config file is optional. Environment variables are preferred. Values read
from environment will override config file values.

The default config path (if you choose to create one) is
`~/.config/eth-partner/config.json`.

### Example Configuration (optional file)

```json
{
  "llm": {
    "google_api_key": "YOUR_GEMINI_API_KEY"
  },
  "wallet_server": {
    "enable": true,
    "rpc_url": "http://127.0.0.1:8545",
    "chain_id": 31337,
    "wallet_file": "/path/to/.wallet.json",
    "gas_limit": null,
    "gas_price": null,
    "listen_address": "127.0.0.1:8546"
  }
}
```

### Web Search (Google CSE)

The `web_search` tool uses Google Programmable Search Engine (CSE) JSON API to
query the public web and return concise results for LLM consumption.

#### Setup steps

1. Create a CSE in the [Programmable Search Control Panel](https://programmablesearchengine.google.com/).
2. Configure it to "Search the entire web".
3. Note the Search Engine ID (aka `cx`).
4. Create/obtain a Google API key with access to the Custom Search API.

#### Required environment variables

Add the following to your `.env` (see `.env.example`):

```env
GOOGLE_SEARCH_API_KEY="your-google-cse-api-key"
GOOGLE_SEARCH_ENGINE_ID="your-cse-engine-id"
```

When both variables are set, the REPL will register the `web_search` tool.
If either is missing, the tool will be unavailable.

#### Tool output format

Note: The tool sets Google CSE `safe=off` to maximize recall, per project
requirements.

The tool returns a JSON string with the following structure:

```json
{
  "total": 2,
  "results": [
    { "index": 1, "title": "...", "url": "...", "snippet": "..." },
    { "index": 2, "title": "...", "url": "...", "snippet": "..." }
  ],
  "provider": "google_cse"
}
```

When there are no results, the tool returns:

```json
{ "total": 0, "results": [], "provider": "google_cse" }
```

### Obtaining API Keys

- **Gemini API Key**: You can obtain a Gemini API key from [Google AI Studio](https://aistudio.google.com/app/apikey).
--

### Running the Application

To run the REPL assistant, navigate to the root of the workspace and use the following command:

```bash
cargo run -p repl
```

This will start the interactive REPL, where you can enter commands or prompts for the AI assistant.

### Commands

- `/help`: Displays a list of available commands.
- `/exit`: Exits the application.

### Usage example (web_search)

With `GOOGLE_SEARCH_API_KEY` and `GOOGLE_SEARCH_ENGINE_ID` set, the agent can
use `web_search` autonomously when needed. Example prompt:

```
"List recent announcements from the Rust project website and pick the most relevant."
```

The tool returns JSON like:

```json
{
  "total": 3,
  "results": [
    { "index": 1, "title": "Rust Blog — Announcing ...", "url": "https://blog.rust-lang.org/...", "snippet": "..." },
    { "index": 2, "title": "Rust 1.xx Released", "url": "https://blog.rust-lang.org/...", "snippet": "..." },
    { "index": 3, "title": "RFC updates", "url": "https://blog.rust-lang.org/...", "snippet": "..." }
  ],
  "provider": "google_cse"
}
```

The LLM chooses by index and cites the selected URL.

## MCP Wallet Integration

When enabled, the REPL starts an embedded MCP wallet server and registers
wallet tools with the agent. This allows you to create/import accounts, query
balances, and send transactions against an Ethereum JSON-RPC endpoint.

### Environment variables

Add the following to your `.env` (see `.env.example`):

```env
# Wallet server enablement
# Default: true (wallet server starts if REPL is run)
# No env var needed to enable by default.

# Ethereum JSON-RPC endpoint
ETH_RPC_URL="http://127.0.0.1:8545"

# Chain ID for transactions (optional; used by tests/e2e with Anvil)
CHAIN_ID=31337

# Path to the wallet file managed by mcp-wallet (optional)
WALLET_FILE="/absolute/path/to/.wallet.json"

# Optional gas parameters
# GAS_LIMIT=21000
# GAS_PRICE=1000000000  # in wei
```

Notes:

- If `ETH_RPC_URL` is not set, the default is `http://127.0.0.1:8545`.
- If `CHAIN_ID`/`WALLET_FILE`/`GAS_LIMIT`/`GAS_PRICE` are not set, they remain
  unset and the wallet/server will pick suitable defaults or rely on node
  values.
- Config file values (when provided) override these env defaults.

### Available wallet tools (examples)

- `new_account` — creates a new Ethereum account.
- `list_accounts` — lists known accounts and nonces.
- `eth_get_balance` — reads the ETH balance of an address.
- `create_tx` / `sign_tx` / `eth_send_signed_transaction` — low-level ops.
- `eth_transfer_eth` — convenience: creates, signs and sends an ETH transfer.
- `eth_get_transaction_info` — fetches transaction by hash.
- `eth_get_transaction_receipt` — fetches transaction receipt and status.

### Running local E2E with Anvil

The test suite includes end-to-end tests that start Foundry's `anvil` and
exercise the wallet tools. In CI, Anvil is installed via the
`foundry-toolchain` action. Locally, install Foundry from
https://book.getfoundry.sh/getting-started/installation and ensure `anvil` is
on your PATH.

Run all tests from workspace root:

```bash
cargo test
```

### Quick start (MCP wallet)

1. Install Foundry and ensure `anvil` is on your PATH.
   - Install instructions: https://book.getfoundry.sh/getting-started/installation

2. Start a local Ethereum dev node:

   ```bash
   anvil
   ```

3. In a separate terminal, set env vars and run the REPL:

   ```bash
   export ETH_RPC_URL="http://127.0.0.1:8545"
   export CHAIN_ID=31337
   # Optionally set WALLET_FILE to persist keys between runs
   # export WALLET_FILE="$HOME/.eth-partner-wallet.json"

   cargo run -p repl
   ```

4. Confirm wallet tools are registered (agent may automatically discover tools).
   If running the embedded flow programmatically, use the helper in
   `repl::start_mcp_wallet_and_client` to list tools. Expected tools include:

   - `new_account`, `list_accounts`
   - `eth_get_balance`, `eth_transfer_eth`
   - `eth_get_transaction_info`, `eth_get_transaction_receipt`

5. Sample transfer flow (programmatic outline):

   - Call `new_account` to create a recipient address.
   - Use `eth_get_balance` to read a funded Anvil account balance.
   - Call `eth_transfer_eth` with `from: "rich"` (an alias you define) or a
     funded address/private key you imported into the wallet file.
   - Poll `eth_get_transaction_receipt` until `status: success`.
   - Verify the recipient balance increased via `eth_get_balance`.

For a complete example, inspect the E2E test in
`repl/tests/e2e_mcp_wallet_tests.rs`.

## Troubleshooting

- __Anvil not found__
  - Ensure Foundry is installed and `anvil` is on your PATH.
  - On Linux/macOS, you may need to `source ~/.bashrc`/`~/.zshrc` after
    installation.

- __Cannot connect to RPC / connection refused__
  - Check `ETH_RPC_URL` points to a running node (e.g. `http://127.0.0.1:8545`).
  - Verify Anvil terminal shows it is listening and no firewall blocks the port.

- __Port conflicts__
  - If `8545` is busy, start Anvil on another port: `anvil --port 8547` and set
    `ETH_RPC_URL="http://127.0.0.1:8547"`.

- __Chain ID mismatch__
  - If you set `CHAIN_ID`, ensure it matches your node’s chain ID. Anvil’s
    default is `31337`.

- __Wallet file issues__
  - If `WALLET_FILE` points to a protected path, run with appropriate
    permissions or choose a different location.
  - If accounts are missing, confirm the JSON file contains expected entries.

- __Tools not visible in REPL__
  - Ensure the wallet server is enabled (default: enabled). If you've disabled
    it in config, re-enable it or remove the override.
  - Check logs for MCP wallet startup errors.

- __Transaction stays pending__
  - Try increasing gas price or re-sending while Anvil is running.
  - Poll `eth_get_transaction_receipt` until success, or inspect Anvil logs.

## Security note

This repository includes a proof-of-concept embedded wallet. Private keys may be
stored unencrypted in a local JSON wallet file for developer convenience. Do
NOT use this in production, do not fund these keys with real assets, and do not
commit wallet files or secrets to source control. Prefer ephemeral keys when
testing, or ensure you secure the wallet file path with appropriate OS
permissions. Future iterations should integrate secure key management and
encryption.
