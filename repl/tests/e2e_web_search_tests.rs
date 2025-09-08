//! End-to-end tests for the web search tool.

// These tests are ignored by default because they require valid API keys
// and make real network requests.
// To run these tests, use the command: `cargo test -- --ignored`

use anyhow::Result;
use repl::agent::ReplAgent;
use repl::handle_line;
use repl::tools::web_search::WebSearchTool;
use rig::client::CompletionClient;
use rig::providers::gemini;
use serde_json::json;

#[tokio::test]
#[ignore]
async fn test_e2e_web_search() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // This test requires both GEMINI_API_KEY and BRAVE_API_KEY to be set.
    // If they are not set, we skip the test.
    let gemini_api_key = match std::env::var("GEMINI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Skipping test_e2e_web_search: GEMINI_API_KEY not set");
            return Ok(());
        }
    };
    let brave_api_key = match std::env::var("BRAVE_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Skipping test_e2e_web_search: BRAVE_API_KEY not set");
            return Ok(());
        }
    };

    // Initialize a real agent with the Gemini client and WebSearchTool
    let client = gemini::Client::new(&gemini_api_key);
    let agent_builder = client
        .agent("gemini-1.5-flash-latest")
        .additional_params(json!({
            "generationConfig": {
                "temperature": 0.2, // Lower temperature for more deterministic output
                "topK": 1,
                "topP": 1,
                "maxOutputTokens": 2048,
                "stopSequences": []
            }
        }))
        .tool(WebSearchTool::new(brave_api_key));

    let agent = Some(ReplAgent::new(agent_builder));

    // Send a prompt that requires a web search for current information
    let response = handle_line("Who is the current CEO of Microsoft?".to_string(), &agent).await?;

    // Assert that the response contains the correct information
    let output = response.expect("Expected a response from the agent");
    assert!(
        output.to_lowercase().contains("nadella"),
        "Response did not contain 'Nadella'"
    );

    Ok(())
}
