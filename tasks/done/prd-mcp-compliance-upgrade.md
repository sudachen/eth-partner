# Product Requirements Document: Full Model Context Protocol Compliance Upgrade

## 1. Introduction/Overview

This document outlines the requirements for refactoring the `mcp-wallet` server to become a fully compliant Model Context Protocol (MCP) server. The current implementation uses a custom stdio loop with JSON messaging, which is compatible with many agentic systems but is not a formal implementation of the protocol. This upgrade will involve integrating the `rmcp` Rust crate to handle the protocol's transport, session management, and message framing, ensuring true interoperability and robustness.

## 2. Goals

1.  **Achieve Full MCP Compliance**: Transition from a compatible stdio server to a fully compliant MCP server by adopting the `rmcp` library.
2.  **Improve Robustness and Maintainability**: Replace the manual stdio loop, command dispatcher, and error handling with the battle-tested, formal structures provided by `rmcp`.
3.  **Guaranteed Interoperability**: Ensure the wallet server can seamlessly connect with any MCP-compliant agent or client without requiring a CLI adapter.
4.  **Modernize the Architecture**: Adopt modern best practices for building MCP tools in Rust by using a dedicated, async-first library.

## 3. Functional Requirements

### 3.1 Server Architecture Refactoring

1.  The server's main entry point (`main.rs`) shall be refactored to initialize and run an `rmcp` server instead of the manual stdio loop.
2.  A new `WalletService` struct shall be created to hold the application state (the `Wallet` instance) and implement the `rmcp::Service` trait.

### 3.2 `rmcp::Service` Trait Implementation

1.  The `WalletService` shall implement the `list_tools` method. This method will be the new authoritative source for the server's capabilities, returning a list of all available wallet commands in the format required by the `rmcp` crate. This will replace the `--get-tool-definition` flag and the `get_tool_definition` MCP command.
2.  The `WalletService` shall implement the `call_tool` method. This method will serve as the new command dispatcher, receiving tool call requests from the `rmcp` server and routing them to the appropriate internal wallet functions.

### 3.3 Command Logic Migration

1.  All existing wallet command logic (`new-account`, `list-accounts`, `create-tx`, `sign-tx`, etc.) shall be made accessible through the `call_tool` method.
2.  The existing custom command parsing and parameter deserialization logic shall be removed and replaced by the argument handling provided by `call_tool`.

### 3.4 Test Suite Upgrade

1.  The integration tests (primarily `mcp_server_tests.rs`) must be rewritten. Instead of writing raw JSON strings to the server's `stdin`, the tests shall use an `rmcp` client to connect to the server and interact with it through the formal protocol.

## 4. Non-Goals (Out of Scope)

1.  Adding any new wallet features (e.g., ERC-20 support, transaction broadcasting).
2.  Supporting any transport layers other than stdio for this phase of the refactoring.
3.  Changing the core logic within the `Wallet` struct itself.

## 5. Technical Considerations

1.  The `rmcp` crate will be added as a core dependency.
2.  The existing `commands` module, including `McpRequest`, `McpResponse`, and the `dispatch_command` function, will be deprecated and removed.
3.  The `--get-tool-definition` CLI flag and the `get_tool_definition` MCP command will be removed, as their functionality is replaced by the `list_tools` method required by the `rmcp::Service` trait.
4.  Error handling will be migrated to use the `rmcp` crate's error types and conventions.
