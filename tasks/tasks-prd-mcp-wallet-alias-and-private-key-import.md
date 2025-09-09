## Relevant Files

- `mcp-wallet/src/wallet.rs` - Core wallet model: accounts, aliases, import and creation logic.
- `mcp-wallet/src/service/mod.rs` - MCP server tools and handlers (e.g., `set_alias`, new `import_private_key`).
- `mcp-wallet/src/main.rs` - Server boot, wallet load/save, handler wiring.
- `mcp-wallet/src/error.rs` - Error types and MCP error mapping.
- `mcp-wallet/src/eth_client.rs` - Ethereum client (used by tests and some tools).
- `mcp-wallet/tests/mcp_server_tests.rs` - Integration tests for MCP tools.
- `mcp-wallet/tests/eth_client_tests.rs` - RPC-related tests.
- `repl/src/tools/mcp_wallet.rs` - REPL tool wrappers for MCP Wallet tools.
- `repl/tests/e2e_mcp_wallet_tests.rs` - E2E tests executing wallet flows via REPL.

### Notes

- New tests for aliasing unknown addresses and private key import will be added without
  breaking existing tests.
- Use `cargo test -p mcp-wallet` to run crate tests and `cargo test -p repl` for REPL E2E.

## Tasks

- [x] 1.0 Extend alias flow to auto-create watch-only accounts
  - [x] 1.1 Update wallet model to support watch-only accounts without private keys (storage-compatible)
  - [x] 1.2 Update `Wallet::add_alias` to create a watch-only account if address is missing
  - [x] 1.3 Ensure `get_signer` returns a proper error for watch-only (no signing) accounts
  - [x] 1.4 Normalize/validate input address and store as `Address` (binary) consistently
  - [x] 1.5 Persist changes and ensure file format remains backward compatible

- [x] 2.0 Implement private key import tool and wallet upgrade path
  - [x] 2.1 Add MCP tool `import_private_key { private_key }` (accept 0x or raw hex)
  - [x] 2.2 Validate private key length/format (32-byte secp256k1, non-zero)
  - [x] 2.3 Derive address; if absent, create signing account (no alias changes)
  - [x] 2.4 If address exists as watch-only, upgrade to signing by attaching key
  - [x] 2.5 If signing account exists, follow current duplicate behavior (no change)
  - [x] 2.6 Ensure storage model and error responses remain unchanged

- [x] 3.0 Validation and normalization for addresses and keys
  - [x] 3.1 Validate addresses via `ethers::types::Address::from_str` and checksum when returning as string
  - [x] 3.2 Add helper for private key normalization (strip 0x, lowercase, length check)
  - [x] 3.3 Add minimal error types/messages reusing existing `WalletError` patterns

- [x] 4.0 Wire MCP endpoints and schema updates (tools manifest)
  - [x] 4.1 Add `import_private_key` handler in `service/mod.rs` with schema params
  - [x] 4.2 Extend `set_alias` handler to auto-create watch-only accounts
  - [x] 4.3 Ensure `list_accounts` reflects watch-only vs signing state in response
  - [x] 4.4 Update server info/instructions minimally if needed

- [ ] 5.0 Add unit tests for wallet logic (alias + import scenarios)
  - [ ] 5.1 Aliasing an unknown address creates a watch-only account
  - [ ] 5.2 Importing a private key creates a signing account if absent
  - [ ] 5.3 Import upgrades an existing watch-only account to signing
  - [ ] 5.4 `get_signer` errors on watch-only accounts
  - [ ] 5.5 Private key validation rejects invalid lengths/hex

- [ ] 6.0 Add integration tests for MCP tools (alias + import)
  - [ ] 6.1 `set_alias` creates watch-only when account missing (MCP call)
  - [ ] 6.2 `import_private_key` adds or upgrades accounts (MCP call)
  - [ ] 6.3 `list_accounts` shows aliases and indicates signing vs watch-only

- [ ] 7.0 Add E2E tests in REPL for alias and import flows
  - [ ] 7.1 REPL invokes `set_alias` on unknown address then confirms via `list_accounts`
  - [ ] 7.2 REPL invokes `import_private_key` then confirms signing capability
  - [ ] 7.3 Ensure no regressions in existing E2E tests

- [ ] 8.0 Documentation updates (README and examples) and finalize
  - [ ] 8.1 Update `mcp-wallet/README.md` with new flows and examples
  - [ ] 8.2 Update `repl/README.md` examples if needed
  - [ ] 8.3 Add notes on validation and watch-only semantics

## Associated PRD

Use @/Projects/eth-partner/tasks/prd-mcp-wallet-alias-and-private-key-import.md
