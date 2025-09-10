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

    // This test requires GEMINI_API_KEY, GOOGLE_SEARCH_API_KEY and GOOGLE_SEARCH_ENGINE_ID to be set.
    // If they are not set, we skip the test.
    let gemini_api_key = match std::env::var("GEMINI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Skipping test_e2e_web_search: GEMINI_API_KEY not set");
            return Ok(());
        }
    };
    let google_search_api_key = match std::env::var("GOOGLE_SEARCH_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Skipping test_e2e_web_search: GOOGLE_SEARCH_API_KEY not set");
            return Ok(());
        }
    };
    let google_search_engine_id = match std::env::var("GOOGLE_SEARCH_ENGINE_ID") {
        Ok(id) => id,
        Err(_) => {
            println!("Skipping test_e2e_web_search: GOOGLE_SEARCH_ENGINE_ID not set");
            return Ok(());
        }
    };

    // Initialize a real agent with the Gemini client and WebSearchTool
    let client = gemini::Client::new(&gemini_api_key);
    let agent_builder = client
        .agent("gemini-2.0-flash")
        .additional_params(json!({
            "generationConfig": {
                "temperature": 0.2, // Lower temperature for more deterministic output
                "topK": 1,
                "topP": 1,
                "maxOutputTokens": 2048,
                "stopSequences": []
            }
        }))
        .tool(WebSearchTool::new(
            google_search_api_key,
            google_search_engine_id,
        ));

    let mut agent = Some(ReplAgent::new(agent_builder));

    // Send a directive prompt that forces returning the web_search tool JSON verbatim
    let response = handle_line(
        "Use the web_search tool to find the current CEO of Microsoft.".to_string(),
        &mut agent,
    )
    .await?;

    // Assert that the response is non-empty (tool executed; format may vary by model)
    let output = response.expect("Expected a response from the agent");
    assert!(
        !output.trim().is_empty(),
        "Agent returned an empty response"
    );

    assert!(output.contains("Nadella"));

    Ok(())
}
