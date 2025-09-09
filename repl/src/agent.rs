//! Core agent logic using the `rig` framework.

use anyhow::Result;
use rig::agent::{Agent, AgentBuilder};
use rig::completion::{CompletionModel, Prompt};
use serde::{Deserialize, Serialize};

/// A struct to represent a single message in the chat history.
#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// A struct to encapsulate the `rig` agent and its functionality.
#[allow(dead_code)]
pub struct ReplAgent<M: CompletionModel> {
    agent: Agent<M>,
    pub history: Vec<ChatMessage>,
}

#[allow(dead_code)]
impl<M: CompletionModel> ReplAgent<M> {
    /// Creates a new `ReplAgent` from an `AgentBuilder`.
    pub fn new(builder: AgentBuilder<M>) -> Self {
        let agent = builder.build();
        Self { agent, history: Vec::new() }
    }

    /// Runs the agent with a given input and returns the response.
    pub async fn run(&self, input: &str) -> Result<String> {
        let response = self.agent.prompt(input).await?;
        Ok(response)
    }
}
