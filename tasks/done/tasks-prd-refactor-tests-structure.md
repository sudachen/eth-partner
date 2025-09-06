## Relevant Files

- `mcp-wallet/src/wallet.rs` - Contains inline tests to be moved.
- `mcp-wallet/tests/wallet_tests.rs` - Destination for tests from `wallet.rs`.
- `mcp-wallet/src/models/transaction.rs` - Contains inline tests to be moved.
- `mcp-wallet/tests/transaction_model_tests.rs` - New test file for `models/transaction.rs`.
- `mcp-wallet/src/transaction/builder.rs` - Contains inline tests to be moved.
- `mcp-wallet/tests/transaction_builder_tests.rs` - New test file for `transaction/builder.rs`.

### Notes

- The goal is to move all `#[cfg(test)]` modules from the `src` directory to the `tests` directory.
- This will convert them from unit tests to integration-style tests, which is a common pattern in Rust for cleaner separation.

## Tasks

- [x] 1.0 Relocate Tests from `wallet.rs`
  - [x] 1.1 Move the test code from the `#[cfg(test)] mod tests` module in `src/wallet.rs` to `tests/wallet_tests.rs`.
  - [x] 1.2 Remove the `tests` module entirely from `src/wallet.rs`.
  - [x] 1.3 Update the imports in `tests/wallet_tests.rs` to pull from the `mcp_wallet` crate (e.g., `use mcp_wallet::wallet::Wallet;`).
  - [x] 1.4 Ensure any helper functions or structs used by the tests are made public or moved to the test file if they are test-specific.

- [x] 2.0 Relocate Tests from `models/transaction.rs`
  - [x] 2.1 Create a new test file: `tests/transaction_model_tests.rs`.
  - [x] 2.2 Move the test code from `src/models/transaction.rs` into the new test file.
  - [x] 2.3 Remove the `tests` module from `src/models/transaction.rs`.
  - [x] 2.4 Add necessary imports to the top of `tests/transaction_model_tests.rs`.

- [x] 3.0 Relocate Tests from `transaction/builder.rs`
  - [x] 3.1 Create a new test file: `tests/transaction_builder_tests.rs`.
  - [x] 3.2 Move the test code from `src/transaction/builder.rs` into the new test file.
  - [x] 3.3 Remove the `tests` module from `src/transaction/builder.rs`.
  - [x] 3.4 Add necessary imports to the top of `tests/transaction_builder_tests.rs`.

- [x] 4.0 Final Verification
  - [x] 4.1 Run `cargo build` to ensure the project compiles without errors.
  - [x] 4.2 Run `cargo test` to ensure the entire test suite passes.
