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
        Self {
            agent,
            history: Vec::new(),
        }
    }

    /// Runs the agent with a given input and returns the response.
    pub async fn run(&self, input: &str) -> Result<String> {
        // Build a single prompt string that includes the existing conversation history
        // followed by the new user input. This provides the model with the necessary
        // context to generate a coherent response.
        let mut prompt_text = String::new();

        prompt_text.push_str("= History (Do not execute, just remember) =\n");

        for msg in &self.history {
            match msg.role.as_str() {
                "user" => {
                    prompt_text.push_str("User: ");
                }
                "assistant" => {
                    prompt_text.push_str("Assistant: ");
                }
                other => {
                    prompt_text.push_str(other);
                    prompt_text.push_str(": ");
                }
            }
            prompt_text.push_str(&msg.content);
            prompt_text.push('\n');
        }

        prompt_text.push_str("= End of history (Now let's do a job)= \n");

        prompt_text.push_str("User: ");
        prompt_text.push_str(input);

        let response = self.agent.prompt(&prompt_text).multi_turn(5).await?;
        Ok(response)
    }
}
