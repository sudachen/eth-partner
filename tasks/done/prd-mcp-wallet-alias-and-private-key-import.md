# PRD: MCP Wallet — Alias Setup and Private Key Import

## Introduction / Overview

This PRD defines two enhancements to the `mcp-wallet` crate (an MCP-style
wallet server for Ethereum-like accounts):

1) Setting an alias for an address must auto-create an account if the address is
   not already present (watch-only, no private key).
2) Importing a private key must add the corresponding account if it does not
   already exist. If a watch-only account with the same address exists, it
   should be upgraded to a signing account by attaching the imported key.

The storage model, error model, and most of the existing MCP tooling remain
unchanged.

## Goals

1. Provide an MCP tool to set an alias that creates a watch-only account if the
   address is not present.
2. Provide an MCP tool to import a private key that creates the account if not
   present, or upgrades an existing watch-only account to signing capability.
3. Maintain current storage model (already implemented) and error handling
   conventions.
4. Validate inputs minimally but correctly (addresses and private keys) without
   introducing a new security/storage scheme.
5. Ensure comprehensive testing: unit, integration (MCP layer), and E2E via the
   `repl` crate.

## User Stories

- As a developer using the MCP wallet, I can set an alias for an Ethereum
  address; if the address is not known, a watch-only account is created so I can
  reference it later by alias.
- As a developer, I can import a private key; if the derived address is not
  known, a new account is created. If it exists as watch-only, it becomes a
  signing account.
- As a developer, I can rely on existing storage behavior and error formatting
  with no new configuration.

## Functional Requirements

1. Alias setup
   1.1. The MCP tool `set_alias { address, alias }` must create a watch-only
        account when `address` is not present.
   1.2. Watch-only accounts must not be usable for signing/sending
        transactions.
   1.3. Address must be validated and normalized (EIP-55 checksum casing) on
        store.
   1.4. Alias uniqueness and collision handling must follow existing wallet
        rules (do not change current behavior).

2. Private key import
   2.1. The MCP tool `import_private_key { private_key }` must accept either
        0x-prefixed or non-prefixed 32-byte hex keys.
   2.2. The key must be validated as a proper 32-byte secp256k1 private key
        (non-zero, within curve order).
   2.3. Derive address from the key; if account does not exist, create it as a
        signing account.
   2.4. If a watch-only account with the same address exists, upgrade it to a
        signing account by attaching the key.
   2.5. If a signing account already exists for the address, follow current
        wallet rules for duplicates (reject/handle as already implemented).
   2.6. Storage model remains unchanged (existing unencrypted dev storage).

3. MCP layer and error handling
   3.1. Reuse standard MCP error response format already used in this repo.
   3.2. Expose `set_alias` and `import_private_key` in the MCP server alongside
        existing tools. Preserve all existing tools and interfaces.

4. Backward compatibility and tests
   4.1. Do not break existing tests or flows.
   4.2. Add new tests for:
        - Aliasing a non-existent address creates a watch-only account.
        - Private key import creates a new account if not present.
        - Private key import upgrades a watch-only account to signing.
        - Input validation happy paths and basic failure cases.
   4.3. Provide E2E tests in `repl/tests` that exercise the MCP calls.

## Non-Goals (Out of Scope)

- Mnemonic/HD wallet import.
- Hardware wallets / multi-sig.
- Encrypted keystore or passphrase flow changes.
- Alias policy changes beyond current implementation.
- Auditing/logging enhancements.

## Design Considerations

- Address format: validate and store in EIP-55 checksum casing to avoid
  ambiguity and support user
  expectations. Accept lower/upper/mixed on input.
- Key format: accept 0x or raw hex; ensure strict 32-byte length.
- Storage: do not change existing file layout or persistence mechanism.
- MCP contract: keep request/response shapes consistent with existing patterns.

## Technical Considerations

- Use existing Ethereum utilities already present in the crate for checksum
  normalization, address derivation, and key validation if available.
- If missing, add minimal helpers without bringing new heavy dependencies.
- Ensure thread-safety and correct state updates for account creation/upgrade in
  the wallet service.

## Success Metrics

- All existing tests pass unchanged.
- New unit, integration, and E2E tests for the flows described above pass.
- MCP clients can set alias for unknown addresses (creating watch-only) and
  import private keys (adding or upgrading accounts) reliably.

## Open Questions

- None at this time, based on user guidance. If existing alias or duplicate-key
  behaviors need refinement beyond "keep current behavior," they will be
  clarified during implementation.
