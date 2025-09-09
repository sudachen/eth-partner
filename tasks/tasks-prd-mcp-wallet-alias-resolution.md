## Relevant Files

- `mcp-wallet/src/service/mod.rs` - Define new MCP tool `resolve_alias` and handler.
- `mcp-wallet/src/wallet.rs` - Alias data structure; may add helper for case-insensitive lookup.
- `mcp-wallet/src/main.rs` - Ensure tool registration and server wiring unchanged after addition.
- `mcp-wallet/tests/mcp_server_tests.rs` - Add unit/integration tests for alias resolution tool.
- `mcp-wallet/tests/wallet_alias_resolution_tests.rs` - Unit tests for case-insensitive alias helper.
- `repl/src/tools/mcp_wallet.rs` - Add REPL client wrapper for `resolve_alias` and auto-resolution hook.
- `repl/src/lib.rs` - Apply pre-resolution of alias for any address-like parameters.
- `repl/tests/e2e_alias_resolution_tests.rs` - New E2E tests for alias usage across operations.
- `mcp-wallet/README.md` - Document the new tool, input/output, and examples.
- `repl/README.md` - Document REPL usage where aliases are accepted transparently.

### Notes

- No ENS, caching, or security. Local-only alias map; case-insensitive lookup.
- The tool returns only `{ address }` on success; otherwise an MCP error (not found).
- Addresses are identical across networks; no chain-specific logic required.

## Tasks

- [x] 1.0 Add `resolve_alias` MCP tool in mcp-wallet
  - [x] 1.1 Define params struct `ResolveAliasParams { alias: String }` in `mcp-wallet/src/service/mod.rs`
  - [x] 1.2 Implement handler `resolve_alias` using wallet lookup (case-insensitive)
  - [x] 1.3 Return EIP-55 checksummed `{ address }` on success
  - [x] 1.4 Return MCP error (invalid params/not found) when alias does not exist
  - [x] 1.5 Ensure tool is exposed via existing `#[tool_router]` pattern

- [x] 2.0 Implement case-insensitive alias resolution logic
  - [x] 2.1 Add wallet helper `resolve_alias_case_insensitive(&self, alias: &str) -> Option<Address>` in `mcp-wallet/src/wallet.rs`
  - [x] 2.2 Reuse existing alias map; do not change storage format or validation rules
  - [x] 2.3 Add focused unit test(s) for the helper if needed (happy path + not found)

- [x] 3.0 Unit tests in mcp-wallet for resolution success and not-found
  - [x] 3.1 Happy path: add alias with mixed case, resolve different case via MCP tool
  - [x] 3.2 Not found: `resolve_alias` returns MCP error with clear message
  - [x] 3.3 Output address is checksummed
  - [x] 3.4 No ENS/network calls are performed (local-only)

- [x] 4.0 None

- [ ] 5.0 E2E tests in REPL validating alias usage in common operations
  - [ ] 5.1 Create `repl/tests/e2e_alias_resolution_tests.rs`
  - [ ] 5.2 Scenario: using LLM agent, set an alias, then ask about of alias's address, and expect success
  - [ ] 5.3 Scenario: use unknown alias and assert a friendly error is returned

- [ ] 6.0 Documentation updates for both crates
  - [ ] 6.1 Update `mcp-wallet/README.md` with the new `resolve_alias` tool (params, output)
  - [ ] 6.2 Update `repl/README.md` to note aliases are accepted transparently for address fields
  - [ ] 6.3 Mention case-insensitive behavior and out-of-scope items (no ENS/caching/security)

## Associated PRD

Use @/Projects/eth-partner/tasks/prd-mcp-wallet-alias-resolution.md
