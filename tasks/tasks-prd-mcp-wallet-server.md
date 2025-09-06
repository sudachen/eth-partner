## Relevant Files

- `src/main.rs` - Main application entry point and CLI interface
- `src/wallet.rs` - Core wallet functionality and data structures
- `src/commands/` - Directory for command implementations
  - `mod.rs` - Command module exports
  - `new_account.rs` - New account creation
  - `set_alias.rs` - Alias management
  - `create_tx.rs` - Transaction creation
  - `sign_tx.rs` - Transaction signing
  - `list_accounts.rs` - Account listing
- `src/models/` - Data models and types
  - `mod.rs` - Model exports
  - `wallet.rs` - Wallet data structure
  - `transaction.rs` - Transaction data structure
- `src/error.rs` - Custom error types
- `src/lib.rs` - Library exports
- `tests/` - Integration tests
- `Cargo.toml` - Project dependencies and configuration

## Tasks

- [x] 1.0 Set up project structure and dependencies
  - [x] 1.1 Initialize new Rust project with `cargo new`
  - [x] 1.2 Add required dependencies to Cargo.toml (ethers, serde, anyhow, thiserror, clap, etc.)
  - [x] 1.3 Set up basic project structure (src/bin, src/lib, tests)
  - [x] 1.4 Configure Rust edition and features in Cargo.toml
  - [x] 1.5 Set up basic error handling with thiserror

- [x] 2.0 Implement core wallet data structures
  - [x] 2.1 Define Wallet struct with accounts and aliases
  - [x] 2.2 Implement Account struct with private key, public key, and nonce
  - [x] 2.3 Add serde serialization/deserialization for all data structures
  - [x] 2.4 Implement validation for aliases (1-20 alphanumeric chars)
  - [x] 2.5 Add tests for wallet data structures

- [x] 3.0 Implement wallet file persistence
  - [x] 3.1 Add functions to load wallet from JSON file
  - [x] 3.2 Add functions to save wallet to JSON file
  - [x] 3.3 Handle file not found case (create new wallet)
  - [x] 3.4 Add file path configuration (default and custom)
  - [x] 3.5 Add tests for file operations

- [x] 4.0 Implement account management commands
  - [x] 4.1 Create new account command
  - [x] 4.2 List accounts command
  - [x] 4.3 Get account by address/alias
  - [x] 4.4 Generate Ethereum addresses from public keys
  - [x] 4.5 Add tests for account management

- [x] 5.0 Implement alias management
  - [x] 5.1 Add alias to address
  - [x] 5.2 Get address by alias
  - [x] 5.3 List all aliases for an address
  - [x] 5.4 Handle alias uniqueness constraints
  - [x] 5.5 Add tests for alias management

- [x] 6.0 Implement transaction creation
  - [x] 6.1 Define Transaction struct with EIP-1559 fields
  - [x] 6.2 Create transaction builder
  - [x] 6.3 Handle network selection (mainnet/devnet)
  - [x] 6.4 Add gas estimation
  - [x] 6.5 Add tests for transaction creation

- [x] 7.0 Implement transaction signing
  - [x] 7.1 Sign transaction with private key
  - [x] 7.2 Handle different transaction types (EIP-1559)
  - [x] 7.3 Update nonce after signing
  - [x] 7.4 Validate transaction before signing
  - [x] 7.5 Add tests for transaction signing

- [ ] 8.0 Set up MCP stdio interface
  - [ ] 8.1 Implement command parsing from stdin
  - [ ] 8.2 Set up command routing
  - [ ] 8.3 Format output as JSON for MCP compatibility
  - [ ] 8.4 Handle command errors gracefully
  - [ ] 8.5 Add tests for stdio interface

- [x] 9.0 Add error handling and validation
  - [x] 9.1 Define custom error types
  - [x] 9.2 Add input validation for all commands
  - [x] 9.3 Handle file I/O errors
  - [x] 9.4 Add error context for debugging
  - [x] 9.5 Add tests for error cases

- [ ] 10.0 Add documentation and examples
  - [ ] 10.1 Document all public APIs
  - [ ] 10.2 Add README with usage examples
  - [ ] 10.3 Document MCP interface specification
  - [ ] 10.4 Add example commands and expected outputs
  - [ ] 10.5 Add integration tests

## Notes

- All code should follow Rust best practices and idiomatic patterns
- Use `anyhow` for application errors and `thiserror` for library errors
- Follow the Ethereum standard for key management and transaction signing
- Ensure all public APIs are well-documented with examples
- Write unit tests for all non-trivial functions
- Use `cargo clippy` and `cargo fmt` to maintain code quality
- Follow semantic versioning for the crate
- Add CI/CD configuration for testing and linting
