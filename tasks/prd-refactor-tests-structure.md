## Product Requirements Document: Refactor Test Structure

### 1. Objective

To improve the project's structure and align with Rust best practices by separating unit tests from the library's source code. This change will move all inline test modules from the `src/` directory to the `tests/` directory, converting them into integration-style tests.

### 2. Background

Currently, several modules within the `mcp-wallet` crate contain inline test modules (`#[cfg(test)] mod tests { ... }`). While this is a valid pattern, it co-locates test code with production code. Moving these tests to the `tests/` directory provides a cleaner separation of concerns, simplifies the library code, and makes the test suite's scope more explicit.

### 3. Requirements

- All code currently inside `#[cfg(test)]` modules within the `src/` directory must be moved to new, separate files within the `tests/` directory.
- The naming convention for the new test files should be `tests/<module_name>_tests.rs` (e.g., tests from `src/wallet.rs` will move to `tests/wallet_tests.rs`).
- The original `#[cfg(test)]` modules must be removed from the source files in `src/`.
- The moved tests must be updated to correctly import necessary types and functions from the `mcp_wallet` crate (e.g., `use mcp_wallet::wallet::Wallet;`).
- The project must successfully compile (`cargo build`) after the refactoring.
- The entire test suite must pass (`cargo test`) after the refactoring.

### 4. Out of Scope

- Modifying the logic of any existing tests.
- Adding new tests or new functionality.
- Changing existing integration tests that are already in the `tests/` directory (e.g., `mcp_server_tests.rs`).
- Modifying any code outside of the test modules, unless required to make items accessible for testing (e.g., changing visibility to `pub(crate)`).
