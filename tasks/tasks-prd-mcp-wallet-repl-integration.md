## Relevant Files

- `repl/src/agent.rs` - Wire MCP wallet startup and tool registration on REPL start.
- `repl/src/config.rs` - Add config for MCP wallet enablement and overrides.
- `repl/src/tools/mcp_wallet.rs` - Bootstrap and transport adapter between agent and mcp-wallet server.
- `mcp-wallet/src/lib.rs` - Existing MCP server crate providing wallet tools (dependency for REPL).
- `repl/tests/e2e_mcp_wallet_tests.rs` - E2E tests: start anvil, run transfer, assert.
- `.github/workflows/rust.yml` - CI updates to install/run `anvil` and execute tests.
- `.env.example` - Document defaults for ETH RPC URL, CHAIN_ID, WALLET_FILE, etc.
- `repl/README.md` - Update docs on configuration, running, and testing MCP wallet.
 - `repl/Cargo.toml` - Add dependency on `mcp-wallet` and any required features.
 - `repl/tests/test_utils/anvil.rs` - Helper to spawn/await/teardown anvil for tests.

### Notes

- E2E tests will programmatically spawn `anvil`, wait until it's ready, and tear it down.
- Config from `.env` acts as defaults; REPL config values override when present.
- On MCP server startup failure, REPL startup should fail (PoC behavior).

## Tasks

- [x] 1.0 Architecture and configuration setup
  - [x] 1.1 Define config knobs in `repl/src/config.rs`:
    - `enable_mcp_wallet: bool` (default true)
    - `eth_rpc_url: Option<String>` (defaults from `.env`)
    - `chain_id: Option<u64>` (defaults from `.env`)
    - `wallet_file: Option<PathBuf>` (path to wallet file managed by mcp-wallet)
    - Optional gas params (e.g., `gas_limit`, `gas_price`)
  - [x] 1.2 Load defaults from `.env` but allow REPL config to override values.
  - [x] 1.3 Update `repl/Cargo.toml` to depend on `mcp-wallet` crate (workspace).
  - [x] 1.4 Add new env keys to `.env.example`:
    - `ETH_RPC_URL`, `CHAIN_ID`, `WALLET_FILE`, optional gas params.
  - [x] 1.5 Decide logging levels and add targeted logs (no secrets).

- [ ] 2.0 Implement in-process MCP transport adapter and server bootstrap
  - [ ] 2.1 Create `repl/src/tools/mcp_wallet.rs` with:
    - `pub async fn start_mcp_wallet_server(cfg: &Config) -> anyhow::Result<ServerHandle>`
    - Graceful shutdown via `ServerHandle::shutdown()`
  - [ ] 2.2 Implement adapter to run the `mcp-wallet` server in-process using
        an in-memory duplex (emulating stdio framing) between agent and server.
  - [ ] 2.3 Ensure the full tool surface from `mcp-wallet` is exposed without filtering.
  - [ ] 2.4 Add robust error messages on startup failure (do not log secrets).

- [ ] 3.0 Wire REPL startup to launch mcp-wallet and register tools with agent
  - [ ] 3.1 In `repl/src/agent.rs`, during startup, if `enable_mcp_wallet` then
        spawn `start_mcp_wallet_server(...)` on the Tokio runtime.
  - [ ] 3.2 Register MCP tools with the agent so they are discoverable and callable.
  - [ ] 3.3 If the server fails to start, fail REPL startup with a clear error.
  - [ ] 3.4 Implement shutdown hook to gracefully stop the MCP server.

- [ ] 4.0 End-to-end tests with anvil bring-up and transfer validation
  - [ ] 4.1 Add `repl/tests/test_utils/anvil.rs` to spawn `anvil` on a free port,
        wait for readiness, and provide teardown.
  - [ ] 4.2 Create `repl/tests/e2e_mcp_wallet_tests.rs`:
    - [ ] 4.2.1 Start `anvil` and set `ETH_RPC_URL`, `CHAIN_ID` for the test.
    - [ ] 4.2.2 Generate a temp wallet file and import a known anvil private key.
    - [ ] 4.2.3 Start the REPL with MCP wallet enabled.
    - [ ] 4.2.4 From the agent, call `mcp-wallet` tools to:
          get balance, get nonce, estimate gas, send transaction.
    - [ ] 4.2.5 Assert receipt status is success and balances changed as expected.
    - [ ] 4.2.6 Ensure proper shutdown of REPL and `anvil`.

- [ ] 5.0 CI workflow updates to install anvil and run tests
  - [ ] 5.1 Update `.github/workflows/rust.yml` to install Foundry (for `anvil`).
  - [ ] 5.2 Cache Rust/Cargo to speed up builds; ensure `cargo test` runs e2e tests.
  - [ ] 5.3 Set necessary env vars in CI or within tests for Anvil URL/chain ID.

- [ ] 6.0 Documentation and examples
  - [ ] 6.1 Update `repl/README.md` with configuration keys and defaults.
  - [ ] 6.2 Add quick start steps: run REPL, confirm tools listed, run sample transfer.
  - [ ] 6.3 Add troubleshooting notes (Anvil not found, port conflicts, etc.).
  - [ ] 6.4 Document security note that this is a PoC; keys in wallet file.

## Associated PRD

Use @tasks/prd-mcp-wallet-repl-integration.md
