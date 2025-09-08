# PRD: Integrate mcp-wallet MCP server into REPL

## Introduction / Overview

We will integrate the `mcp-wallet` MCP (Model Context Protocol) server into the
`repl` crate so it starts as an in-process async task during REPL startup and is
available to the LLM agent. This enables the agent to use Ethereum wallet tools
(balance, nonce, send transaction, estimate gas, etc.) provided by the
`mcp-wallet` server, against a local Anvil node for end-to-end testing.

## Goals

1. Start the `mcp-wallet` MCP server as an in-process Tokio task on REPL start.
2. Use stdio as the MCP transport between the agent and the in-process server.
3. Expose the full tool surface of the existing `mcp-wallet` MCP server to the
   agent without additional filtering.
4. Provide configuration via `.env` for defaults with REPL config values able to
   override.
5. Provide robust end-to-end tests that bring up `anvil` automatically, run a
   funded test transfer, and verify results.
6. Ensure the agent can enumerate and call the mcp-wallet tools once REPL starts.

## User Stories

- As an LLM agent running in the REPL, I can list available MCP tools and see
  the `mcp-wallet` tools exposed.
- As a user/developer, I can start the REPL and automatically have
  `mcp-wallet` server available to the agent without manual steps.
- As a test suite, I can start `anvil` programmatically, fund accounts, execute
  a transfer through `mcp-wallet`, and assert balances and receipts.

## Functional Requirements

1. The REPL must spawn the `mcp-wallet` MCP server as an in-process Tokio task
   during startup. Startup should register the server with the agent so tools
   are discoverable immediately.
2. The MCP transport must be stdio between REPL agent and the in-process server.
3. The agent must expose the full existing toolset from `mcp-wallet` with no
   reduction in scope.
4. Configuration must support both `.env` and REPL config, with REPL config
   overriding `.env` values when both are present.
5. E2E tests must programmatically launch `anvil`, wait until ready, run a
   transfer using `mcp-wallet` tools, and validate expected state (balances,
   receipt status). Tests must also tear down `anvil` afterward.
6. On `mcp-wallet` MCP server startup failure, the REPL must fail startup and
   surface a clear error (proof-of-concept behavior).
7. Documentation (`repl/README.md`) must be updated to describe configuration,
   running the REPL with the `mcp-wallet` integration, and testing guidance.

## Non-Goals (Out of Scope)

- Hardening for production security: this is a PoC. Secrets management,
  keyring integration, and advanced auth are out of scope.
- Advanced transport options (e.g., TCP) beyond stdio.
- Custom curation or filtering of tools beyond what `mcp-wallet` already
  implements.

## Design Considerations

- Placement: Integrate adapter/bootstrap logic within `repl/src/tools/` (flat),
  with minimal wiring in `repl/src/agent.rs` for startup and registration.
- Async runtime: Use the existing Tokio runtime within the REPL. The MCP server
  task should be spawned with appropriate cancellation on REPL shutdown.
- Resilience: On startup failure, fail REPL startup (per acceptance). Logging
  should be descriptive but must not log secrets.
- Wallet/keys: Managed by `mcp-wallet` in a wallet file. Treat as non-secure
  PoC, do not log private keys.

## Technical Considerations

- Dependencies: Ensure `mcp-wallet` crate is available and can be depended upon
  by `repl` as a workspace member. Confirm compatible versions.
- Testing: E2E tests should mirror `mcp-wallet` tests for Anvil bring-up.
  Consider using a helper to spawn `anvil`, wait on its readiness, and clean up.
- CI: Update `.github/workflows/rust.yml` to install `anvil` (via Foundry or a
  direct download) so tests run in CI.
- Configuration keys to support (examples):
  - ETH_RPC_URL (default Anvil URL)
  - CHAIN_ID
  - DEFAULT_FROM / DEFAULT_TO (optional)
  - WALLET_FILE path (consumed by mcp-wallet)
  - GAS params (optional)

## Success Metrics

- REPL startup launches the MCP server and the agent lists `mcp-wallet` tools.
- A full E2E test on CI starts `anvil`, performs a transfer via `mcp-wallet`,
  and asserts balances and receipts successfully.
- Local developer experience: Minimal setup; REPL runs with `.env` defaults,
  override via REPL config when needed.

## Open Questions

- None currently. Future iterations may add security hardening, TCP transport,
  and tool filtering.
