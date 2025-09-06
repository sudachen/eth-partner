# RMCP Crate Usage Guidelines

This document outlines key insights and best practices for using the `rmcp` crate, based on the refactoring of the `mcp-wallet` server.

## 1. Dependency Management

- **Specify a Version**: Always pin to a specific version of `rmcp` to ensure a stable API. The version that proved to be working is `0.6.3`.
- **Schemars Compatibility**: `rmcp v0.6.3` requires `schemars v1.0.4`. Ensure this version is specified in `Cargo.toml` to avoid trait implementation conflicts.

  ```toml
  rmcp = { version = "0.6.3", features = ["macros"] }
  schemars = "1.0.4"
  ```

## 2. Service Implementation

- **The Definitive `#[tool_router]` Pattern**: The `#[tool_router]` attribute is applied to the `impl` block of the handler struct. This macro processes the methods within and generates a static method `Self::tool_router(state: Self) -> ToolRouter<Self>`.

- **Struct Composition**: The handler struct must contain a field, typically named `tool_router`, of type `ToolRouter<Self>`.

- **Initialization**: The `new()` function is responsible for creating the handler's state. It then calls the generated static `Self::tool_router(self)` method to construct the router and assign it to the `tool_router` field.

- **Tool Methods**: Individual methods within the `#[tool_router]`-annotated `impl` block that should be exposed as tools must be annotated with `#[tool]`.

  ```rust
  use rmcp::{handler::server::router::ToolRouter, tool, tool_router};

  #[derive(Clone)]
  pub struct MyHandler {
      tool_router: ToolRouter<Self>,
      // ... other state fields
  }

  #[tool_router]
  impl MyHandler {
      pub fn new(/*...initial state...*/) -> Self {
          Self {
              tool_router: Self::tool_router() // auto build router
              // ... initialize other state
          }
      }

      #[tool]
      async fn my_tool(&self) -> Result<CallToolResult, ErrorData> { /* ... */ }
  }
  ```

## 3. Starting the Server

- **Serving the Handler**: To start the server, use the `ServiceExt::serve` method provided by the `rmcp` crate. This method takes a transport object as an argument.

  ```rust
  // In main.rs or where the server is set up:
  let handler = MyHandler::new();
  // The `ServiceExt` trait provides the `serve` method.
  let service = handler.serve(transport).await?;
  service.wait().await?;
  ```

## 4. Tool Parameters

- **Parameter Wrapper**: All tool function arguments that represent the tool's parameters must be wrapped in the `rmcp::handler::server::wrapper::Parameters<T>` type.

- **Accessing Parameters**: The `Parameters<T>` type is a tuple struct. To access the inner parameter struct, use the `.0` index (e.g., `params.0.field`).

- **Derive `JsonSchema`**: The structs used for tool parameters must derive `schemars::JsonSchema` in addition to `serde::Deserialize`.

  ```rust
  #[derive(Deserialize, Debug, schemars::JsonSchema)]
  struct MyToolParams { /* ... */ }
  ```

## 5. Return Types and Error Handling

- **Result Type**: The return type for tool functions should be `Result<CallToolResult, rmcp::model::ErrorData>`, where `CallToolResult` is a struct that wraps the output.

- **Accessing Structured Results**: The `CallToolResult` struct has a `structured_content` field of type `Option<Value>`. This field contains the primary JSON result of the tool call, which can be used directly.

  ```rust
  // In the client/test code
  let result = client.call_tool(...).await?;
  let json_value = result.structured_content.unwrap();
  assert_eq!(json_value["status"], "ok");
  ```

- **Wrapping Results**: The successful JSON value should be wrapped within a `CallToolResult` using the `CallToolResult::structured()` method. This method returns a `CallToolResult` value directly.

  ```rust
  use rmcp::model::CallToolResult;
  use serde_json::json;

  let my_json_value = json!({ "status": "ok" });
  let result = CallToolResult::structured(my_json_value);
  Ok(result)
  ```

- **Error Type**: The error part of the `Result` should be `rmcp::model::ErrorData`. When creating an `ErrorData` instance, use the static methods like `ErrorData::internal_error(message, None)` and `ErrorData::invalid_params(message, None)`.

  ```rust
  fn to_internal_error<E: std::fmt::Display>(e: E) -> rmcp::model::ErrorData {
      rmcp::model::ErrorData::internal_error(e.to_string(), None)
  }
  ```

## 6. Creating an MCP Client

- **Use `ServiceExt::serve` on the Unit Type**: The idiomatic way to create a client is to call the `serve` method on the unit type `()`. The `rmcp` crate provides a `ClientHandler` implementation for `()`, which acts as a default, stateless client handler. The `serve` method returns a client instance directly.

- **No Client Import Needed**: You do not need to import a `Client` struct. The compiler will infer the correct type of the returned client object.

  ```rust
  use rmcp::ServiceExt;
  use tokio::io::duplex;

  let (client_stream, _server_stream) = duplex(1024);

  // The `serve` method on the unit type returns a client instance.
  let client = ().serve(client_stream).await?;

  // Now you can use the client to make calls.
  client.list_tools(ListToolsRequest::default()).await?;
  ```
