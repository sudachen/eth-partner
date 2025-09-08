//! Integration tests for the REPL application.
#![allow(clippy::manual_async_fn)]

use anyhow::Result;
use repl::agent::ReplAgent;
use repl::handle_line;
use rig::agent::AgentBuilder;
use rig::completion::{
    AssistantContent, CompletionError, CompletionModel, CompletionRequest, CompletionResponse,
    Message, Prompt, PromptError, Usage,
};
use rig::message::Text;
use rig::one_or_many::OneOrMany;
use rig::streaming::StreamingCompletionResponse;
use std::future::{Future, IntoFuture};

/// A mock completion model for testing.
#[derive(Clone)]
struct MockCompletionModel;

impl Prompt for MockCompletionModel {
    fn prompt(
        &self,
        _prompt: impl Into<Message> + Send,
    ) -> impl IntoFuture<Output = Result<String, PromptError>, IntoFuture: Send> {
        async { Ok("mocked response".to_string()) }
    }
}

impl CompletionModel for MockCompletionModel {
    type Response = OneOrMany<AssistantContent>;
    type StreamingResponse = (); // Use unit type to satisfy GetTokenUsage

    fn completion(
        &self,
        _request: CompletionRequest,
    ) -> impl Future<Output = Result<CompletionResponse<Self::Response>, CompletionError>> + Send
    {
        async {
            let text = Text {
                text: "mocked response".to_string(),
            };
            let content = AssistantContent::Text(text);

            Ok(CompletionResponse {
                choice: OneOrMany::one(content.clone()),
                usage: Usage::default(),
                raw_response: OneOrMany::one(content),
            })
        }
    }

    fn stream(
        &self,
        _request: CompletionRequest,
    ) -> impl Future<
        Output = Result<StreamingCompletionResponse<Self::StreamingResponse>, CompletionError>,
    > + Send {
        async { unimplemented!() }
    }
}

#[tokio::test]
async fn test_handle_line_help_command() {
    let agent: Option<ReplAgent<MockCompletionModel>> = None;
    let output = handle_line("/help".to_string(), &agent).await.unwrap();
    assert_eq!(output, Some("Commands: /exit, /help".to_string()));
}

#[tokio::test]
async fn test_handle_line_exit_command() {
    let agent: Option<ReplAgent<MockCompletionModel>> = None;
    let output = handle_line("/exit".to_string(), &agent).await.unwrap();
    assert_eq!(output, None);
}

#[tokio::test]
async fn test_handle_line_prompt_agent() {
    let agent_builder = AgentBuilder::new(MockCompletionModel);
    let agent = Some(ReplAgent::new(agent_builder));

    let output = handle_line("hello".to_string(), &agent).await.unwrap();
    assert_eq!(output, Some("Response: mocked response\n".to_string()));
}

#[tokio::test]
async fn test_handle_line_no_agent() {
    let agent: Option<ReplAgent<MockCompletionModel>> = None;
    let output = handle_line("some prompt".to_string(), &agent)
        .await
        .unwrap();
    assert_eq!(
        output,
        Some(
            "LLM agent not initialized. Please set GEMINI_API_KEY in your environment or config."
                .to_string()
        )
    );
}
