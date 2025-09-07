//! Brave Search API tool implementation.

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;

// --- Error Type ---
#[derive(Error, Debug)]
pub enum WebSearchError {
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Brave API error: {status}: {message}")]
    Api { status: u16, message: String },
}

// --- Argument and Output Structs ---
#[derive(Deserialize, Debug)]
pub struct WebSearchArgs {
    query: String,
}

// --- Tool Struct ---
#[derive(Clone)]
pub struct WebSearchTool {
    client: reqwest::Client,
    api_key: String,
}

impl WebSearchTool {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }
}

// --- API Response Structs ---
#[derive(Deserialize, Debug)]
struct BraveSearchResponse {
    web: Option<WebSearchResults>,
}

#[derive(Deserialize, Debug)]
struct WebSearchResults {
    results: Vec<SearchResult>,
}

#[derive(Deserialize, Debug)]
struct SearchResult {
    title: String,
    url: String,
    description: String,
}

// --- Tool Trait Implementation ---
impl Tool for WebSearchTool {
    const NAME: &'static str = "web_search";

    type Error = WebSearchError;
    type Args = WebSearchArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Searches the web for a given query and returns a list of results."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query."
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let response = self
            .client
            .get("https://api.search.brave.com/res/v1/web/search")
            .header("Accept", "application/json")
            .header("X-Subscription-Token", &self.api_key)
            .query(&[("q", &args.query)])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(WebSearchError::Api { status, message });
        }

        let search_response: BraveSearchResponse = response.json().await?;

        let results = match search_response.web {
            Some(web) => web.results,
            None => return Ok("No web results found.".to_string()),
        };

        let formatted_results = results
            .iter()
            .take(5) // Limit to 5 results to avoid overwhelming the context window
            .map(|result| {
                format!(
                    "Title: {}\nURL: {}\nDescription: {}\n",
                    result.title, result.url, result.description
                )
            })
            .collect::<Vec<String>>()
            .join("\n---\n");

        Ok(formatted_results)
    }
}
