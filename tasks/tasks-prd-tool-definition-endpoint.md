## Relevant Files

- `mcp-wallet/src/main.rs` - To handle the new `--get-tool-definition` command-line argument.
- `mcp-wallet/src/commands/mod.rs` - To handle the new `get_tool_definition` MCP command.
- `mcp-wallet/src/commands/tool_definition.rs` - A new file to house the logic for generating the tool definition.
- `mcp-wallet/tests/tool_definition_tests.rs` - A new test file for the tool definition functionality.

### Notes

- The implementation should be modular to make it easy to add new command definitions in the future.
- The JSON structure must be strictly adhered to for compatibility with AI agentic systems.

## Tasks

- [ ] 1.0 Define Tool Definition Data Structures
  - [ ] 1.1 Create a new module `mcp-wallet/src/commands/tool_definition.rs`.
  - [ ] 1.2 In the new module, define Rust structs (`ToolDefinition`, `Function`, `Parameters`, etc.) that can be serialized into the target JSON structure.
  - [ ] 1.3 Use `serde` attributes (`rename_all`, `skip_serializing_if`) to ensure the JSON output matches the required format.

- [ ] 2.0 Implement Tool Definition Generator
  - [ ] 2.1 Create a public function `generate_tool_definition()` in the `tool_definition` module.
  - [ ] 2.2 Inside this function, construct the complete `ToolDefinition` object.
  - [ ] 2.3 For each available command (`new-account`, `create-tx`, etc.), create a corresponding `Function` struct with its name, description, and parameters.
  - [ ] 2.4 Ensure the descriptions are clear and helpful for an AI agent.

- [ ] 3.0 Add `--get-tool-definition` CLI Flag
  - [ ] 3.1 In `mcp-wallet/src/main.rs`, update the `clap` argument parsing to include a `--get-tool-definition` flag.
  - [ ] 3.2 If the flag is present, call `generate_tool_definition()`, serialize the result to JSON, print it to stdout, and exit the program successfully.
  - [ ] 3.3 Ensure no other part of the application (like wallet loading or the MCP loop) runs when this flag is used.

- [ ] 4.0 Add `get_tool_definition` MCP Command
  - [ ] 4.1 In `mcp-wallet/src/commands/mod.rs`, add a new match arm to the `dispatch_command` function for `get_tool_definition`.
  - [ ] 4.2 When the command is received, call `generate_tool_definition()` and return the result within a standard `McpResponse::success()` wrapper.

- [ ] 5.0 Add Tests
  - [ ] 5.1 Create a new test file `mcp-wallet/tests/tool_definition_tests.rs`.
  - [ ] 5.2 Add a unit test to verify that `generate_tool_definition()` produces a correctly structured and populated JSON object.
  - [ ] 5.3 Add an integration test to verify that running `mcp-wallet --get-tool-definition` prints the JSON and exits.
  - [ ] 5.4 Add a test to the `mcp_server_tests.rs` to verify the `get_tool_definition` command works correctly in an interactive session.
