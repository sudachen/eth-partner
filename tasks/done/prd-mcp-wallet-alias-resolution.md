# PRD: MCP Wallet Alias Resolution Tool

## Introduction / Overview

We will add a new MCP tool to the `mcp-wallet` server named `resolve_alias`.
This tool resolves a user-provided alias to an Ethereum address known by the
wallet. The REPL will use this tool automatically so that any operation that
accepts an address can transparently accept a known alias instead.

This is a proof-of-concept feature: no ENS, no caching, and no security layer.

## Goals

- Enable a `resolve_alias` MCP tool that returns a single address for a given
  alias when it exists.
- Support case-insensitive alias resolution while preserving existing alias
  storage and validation rules.
- Integrate resolution into the REPL so that any address-accepting operation
  benefits (send, approve, watch, etc.).
- Provide unit tests (mcp-wallet) and E2E tests (repl) validating the behavior.

## User Stories

- As a user, I can use a friendly alias instead of a hex address anywhere the
  REPL expects an address, and the system will resolve it to the correct
  address automatically.
- As a user, when I input an unrecognized alias, I get a clear error telling me
  that the alias is not found.

## Functional Requirements

1. The `mcp-wallet` server exposes a new MCP tool `resolve_alias`.
   - Input: `{ alias: string }`.
   - Behavior: Perform a case-insensitive lookup among aliases known to the
     wallet and return the corresponding address if a unique match is found.
   - Output on success: `{ address: string }` (EIP-55 checksummed).
   - Output on failure: an MCP error indicating alias not found.
2. Resolution is strictly local to the wallet (no ENS, no network IO).
3. The REPL automatically invokes `resolve_alias` when the user passes a
   non-empty, non-hex, address-like field.
   - If the input is a valid hex address, pass it through unchanged.
   - If the input is not a valid hex address, treat it as an alias and call
     `resolve_alias`.
   - On not found: surface a clear error to the user for that operation.
4. Case-insensitive resolution must not break existing alias assignment rules.
   - Existing `set_alias` validation rules remain as-is.
   - No duplicate aliases with different case should be allowed in future, but
     this POC will only read case-insensitively; it does not change storage.
5. Testing:
   - Unit tests in `mcp-wallet` verifying case-insensitive resolution and
     not-found error.
   - E2E tests in `repl` verifying that address parameters are resolved via the
     tool for common operations.

## Non-Goals (Out of Scope)

- ENS resolution or any external name service.
- Caching, performance optimization, or TTLs.
- Multi-tenant security, authentication, or authorization.
- Chain-specific alias behavior (aliases are global/same on all networks).

## Design Considerations

- `mcp-wallet` currently stores aliases in a `HashMap<String, Address>`.
  Case-insensitive resolution can be implemented by scanning the map and
  comparing aliases using a common normalization (e.g., lowercasing both the
  input and candidate). This avoids breaking storage format or existing rules.
- MCP tool discovery and JSON schema are generated via `rmcp` macros; the new
  tool should follow the existing pattern in `mcp-wallet/src/service/mod.rs`.
- REPL integration should be a minimal pre-processing layer that attempts
  resolution only when the field is not already a valid hex address.

## Technical Considerations

- Return EIP-55 checksummed addresses from the tool for consistency with other
  endpoints.
- Maintain explicit and descriptive error messages for "not found" and
  "invalid input" to keep the UX clear.

## Success Metrics

- All unit tests in `mcp-wallet` for alias resolution pass.
- All new E2E tests in `repl` for alias usage in address fields pass.
- REPL users can seamlessly use aliases in operations that require addresses.

## Open Questions

- None at this time; confirmed: local-only aliases, case-insensitive resolution,
  return just the address on success, and add unit + E2E tests.
