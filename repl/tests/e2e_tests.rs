//! End-to-end tests for the REPL application.
//!
//! To run these tests, you must have a valid `GEMINI_API_KEY` set in your environment.
//! You can run the tests using the following command:
//! `cargo test -- --ignored`

// These tests are ignored by default because they require a valid GEMINI_API_KEY
// and make real network requests to the Gemini API.
// To run these tests, use the command: `cargo test -- --ignored`

use anyhow::Result;
use repl::agent::ReplAgent;
use repl::handle_line;
use rig::client::CompletionClient;
use rig::providers::gemini;
use serde_json::json;

#[tokio::test]
#[ignore]
async fn test_e2e_gemini_prompt() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // This test requires the GEMINI_API_KEY environment variable to be set.
    // If it's not set, we skip the test.
    let api_key = match std::env::var("GEMINI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Skipping test_e2e_gemini_prompt: GEMINI_API_KEY not set");
            return Ok(());
        }
    };

    // Initialize a real agent with the Gemini client
    let client = gemini::Client::new(&api_key);
    let agent_builder = client
        .agent("gemini-1.5-flash-latest")
        .additional_params(json!({
            "generationConfig": {
                "temperature": 0.9,
                "topK": 1,
                "topP": 1,
                "maxOutputTokens": 2048,
                "stopSequences": []
            }
        }));
    let agent = Some(ReplAgent::new(agent_builder));

    // 1. Send the 'Say Hi' prompt
    let response = handle_line("Say Hi".to_string(), &agent).await?;

    // 3. Assert that the response contains 'Hi'
    let output = response.expect("Expected a response from the agent");
    assert!(output.contains("Hi"), "Response did not contain 'Hi'");

    // 4. Send the '/exit' command
    let exit_response = handle_line("/exit".to_string(), &agent).await?;

    // 5. Assert that the application signals to exit
    assert!(exit_response.is_none(), "Expected None to signal exit");

    Ok(())
}
