# Product Requirements Document: Self-Describing Tool Definition

## 1. Introduction/Overview

To improve integration with AI agentic systems like Gemini and Claude, the `mcp-wallet` server needs to become self-describing. This involves implementing a new command that returns a machine-readable definition of all its available tools (commands), parameters, and descriptions. This eliminates the need for developers to maintain a static YAML or JSON configuration file, making the server the single source of truth for its own API.

## 2. Goals

1.  **Single Source of Truth**: The wallet server's binary should be the only place where its command API is defined.
2.  **Simplified Integration**: Make it easier for administrators of AI agentic systems to integrate and maintain the `mcp-wallet` tool.
3.  **Dynamic Updates**: Ensure that when new commands are added to the wallet, the tool definition is automatically updated.
4.  **Adherence to Convention**: Follow the established best practice for high-quality MCP servers by providing a self-description mechanism.

## 3. Functional Requirements

### 3.1 `get_tool_definition` Command

1.  A new command, `get_tool_definition`, shall be added to the MCP interface.
2.  This command shall take no parameters.
3.  When invoked, the command shall return a JSON object that describes all available public commands (e.g., `new-account`, `create-tx`, `sign-tx`).
4.  The structure of the JSON response must be compatible with the tool definition format expected by modern AI agentic systems.

### 3.2 Command-line Argument

1.  A new command-line argument, `--get-tool-definition`, shall be added to the `mcp-wallet` binary.
2.  When the binary is run with this flag, it should print the JSON tool definition to standard output and then immediately exit.
3.  This provides a simple, scriptable way for external systems to retrieve the tool definition without starting a full interactive session.

### 3.3 JSON Output Structure

The JSON output must contain a list of functions, where each function includes:
- `name`: The name of the command.
- `description`: A natural language description of what the command does.
- `parameters`: An object describing the parameters the command accepts, including their type and description, following a structure similar to the OpenAPI Schema.

**Example Structure:**
```json
{
  "tool": {
    "name": "eth_wallet_manager",
    "description": "Manages an Ethereum wallet...",
    "functions": [
      {
        "name": "new_account",
        "description": "Generates a new Ethereum keypair...",
        "parameters": {
          "type": "object",
          "properties": {
            "alias": {
              "type": "string",
              "description": "A human-readable alias..."
            }
          }
        }
      }
      // ... other functions
    ]
  }
}
```

## 4. Non-Goals (Out of Scope)

1.  Implementing a full OpenAPI or JSON Schema validation system within the wallet.
2.  Changing the existing MCP command-response flow for regular operations.
3.  Adding tool discovery mechanisms beyond the `get_tool_definition` command and `--get-tool-definition` flag.

## 5. Technical Considerations

1.  The tool definition should be constructed in a modular way in the Rust code, making it easy to add new command definitions as the wallet evolves.
2.  The `clap` crate, which is already a dependency, should be used to handle the new `--get-tool-definition` command-line argument.
3.  The `serde_json` crate will be used to construct the final JSON output.
