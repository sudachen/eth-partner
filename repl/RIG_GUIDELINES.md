# Guidelines for Working with the `rig` Crate

This document captures key insights and best practices for working with the `rig` crate, based on challenges encountered during development.

## 1. Agent Interaction

The primary way to get a response from a `rig::agent::Agent` is by using the `prompt` method from the `rig::completion::Prompt` trait. The `Agent` struct implements this trait, providing a high-level interface for interacting with the language model.

```rust
// Example from repl/src/agent.rs
use rig::completion::Prompt;

// ...

let response = self.agent.prompt(input).await?;
```

## 2. Mocking for Tests

When writing tests for code that interacts with a `rig::agent::Agent`, you must provide a mock implementation of the `rig::completion::CompletionModel` trait. Here are the key considerations:

### `completion` Method Implementation

The `Agent::prompt` method internally calls the `completion` method on the `CompletionModel`. Therefore, your mock must provide a valid implementation for `completion`, even if your test only directly calls `prompt`.

### `CompletionResponse` Construction

The `completion` method must return a `Result` containing a `CompletionResponse`. This struct has two important fields:

- `choice`: This field is of type `OneOrMany<AssistantContent>`.
- `usage`: This field is of type `Usage`.

To construct a simple text response, you must build up the types as follows:

1.  Create a `rig::message::Text` struct.
2.  Wrap it in the `rig::completion::AssistantContent::Text` enum variant.
3.  Wrap the `AssistantContent` in `OneOrMany::one()`.

```rust
// Example from repl/tests/repl_tests.rs
use rig::message::Text;
use rig::completion::{AssistantContent, CompletionResponse, Usage};
use rig::one_or_many::OneOrMany;

// ...

let text = Text { text: "mocked response".to_string() };
let content = AssistantContent::Text(text);

Ok(CompletionResponse {
    choice: OneOrMany::one(content.clone()),
    usage: Usage::default(),
    raw_response: OneOrMany::one(content),
})
```

## 3. Asynchronous Traits

The `rig` crate makes extensive use of asynchronous traits, which can be complex to mock. Pay close attention to the following:

- **Method Signatures**: Ensure that the method signatures in your mock implementation match the trait definitions precisely. This often requires manually specifying the `Future` return types (e.g., `Pin<Box<dyn Future<...>>>` or `impl Future<...>`).
- **`Send` Bounds**: Be mindful of `Send` bounds on `Future`s and `IntoFuture`s, as these are often required for the types to be safely sent across threads.

## 4. API and Module Structure

The `rig` crate has a nested module structure, and some components are not part of the public API. When you encounter unresolved import errors, consult the official documentation or the crate's `lib.rs` file to determine the correct public path for a given type. Avoid trying to import from private modules (e.g., `rig::agent::tool`).

## 5. Provider-Specific Configurations

Some LLM providers require specific configuration parameters that are not part of the standard `AgentBuilder` methods. For example, the Gemini provider requires a `generationConfig` object.

To provide these configurations, use the `additional_params` method on the `AgentBuilder`. This method accepts a `serde_json::Value`.

### Example: Gemini `generationConfig`

When using the Gemini provider, you must provide a `generationConfig`. If you don't, you will receive a `JsonError: missing field 'generationConfig'` error from the API.

```rust
// Example from repl/tests/e2e_tests.rs
use rig::client::CompletionClient;
use serde_json::json;

// ...

let agent_builder = client.agent("gemini-2.0-flash").additional_params(json!({
    "generationConfig": {
        "temperature": 0.9,
        "topK": 1,
        "topP": 1,
        "maxOutputTokens": 2048,
        "stopSequences": []
    }
}));
