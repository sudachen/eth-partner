## Relevant Files

- `mcp-wallet/Cargo.toml` - To add the `rmcp` dependency.
- `mcp-wallet/src/main.rs` - To be refactored to run the `rmcp` server.
- `mcp-wallet/src/service/mod.rs` - A new module to house the `WalletService` and its `rmcp::Service` implementation.
- `mcp-wallet/src/lib.rs` - To declare the new `service` module and remove the old `commands` module.
- `mcp-wallet/tests/mcp_server_tests.rs` - To be rewritten to use an `rmcp` client for testing.
- `mcp-wallet/src/commands/` - This entire directory will be removed.

### Notes

- This is a significant refactoring. The core goal is to replace our custom command handling with the formal structures of the `rmcp` crate.
- Error handling will need to be carefully migrated to fit the `rmcp::service::ServiceError` type.

## Tasks

- [x] 1.0 Project Setup and Restructuring
  - [x] 1.1 Add the `rmcp` crate as a dependency in `mcp-wallet/Cargo.toml`.
  - [x] 1.2 Create a new module `mcp-wallet/src/service/mod.rs`.
  - [x] 1.3 In `mcp-wallet/src/lib.rs`, declare the new `pub mod service`.
  - [x] 1.4 Delete the entire `mcp-wallet/src/commands/` directory.
  - [x] 1.5 In `mcp-wallet/src/lib.rs`, remove the `pub mod commands` declaration.
  - [x] 1.6 Delete the obsolete test file `mcp-wallet/tests/tool_definition_tests.rs`.

- [x] 2.0 Implement `WalletService` and `rmcp::Service` Trait
  - [x] 2.1 In `service/mod.rs`, define a `WalletService` struct that holds an `Arc<Mutex<Wallet>>`.
  - [x] 2.2 Implement the `#[async_trait]` for `rmcp::Service` on `WalletService`.
  - [x] 2.3 Implement the `list_tools` method, returning a `Vec<rmcp::model::Tool>` that describes all wallet commands.
  - [x] 2.4 Implement the `call_tool` method, which will act as the new command dispatcher.
  - [x] 2.5 Inside `call_tool`, use a `match` statement on the tool name to route to the correct logic.
  - [x] 2.6 For each command, parse the `arguments` from the request and call the appropriate `wallet` method.
  - [x] 2.7 Convert the `Result` from the wallet methods into a `rmcp::service::Result<CallToolResult>`.

- [x] 3.0 Refactor `main.rs` to Run the MCP Server
  - [x] 3.1 Remove the `clap` dependency and argument parsing for `--get-tool-definition`.
  - [x] 3.2 Remove the existing stdio read loop from the `main` function.
  - [x] 3.3 In `main`, after loading the `Wallet`, create an instance of `WalletService`.
  - [x] 3.4 Create an `rmcp` stdio transport using `(tokio::io::stdin(), tokio::io::stdout())`.
  - [x] 3.5 Use `rmcp::service::serve_server` to start the server with the `WalletService` and transport.

- [x] 4.0 Upgrade Integration Tests
  - [x] 4.1 In `mcp_server_tests.rs`, remove the existing test logic that uses `std::process::Command`.
  - [x] 4.2 Rewrite the `test_full_workflow` to start the `mcp-wallet` server in a background task.
  - [x] 4.3 In the test, create an `rmcp` client that connects to the server's stdio streams.
  - [x] 4.4 Use the client's `list_tools` and `call_tool` methods to interact with the server and assert the responses.


# Note 

1. RoleServer is in the root rmcp module
2. 