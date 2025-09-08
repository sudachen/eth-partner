//! Web Search API tool implementation (provider to be configured).

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;

// --- Error Type ---
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum WebSearchError {
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Search API error: {status}: {message}")]
    Api { status: u16, message: String },
    #[error("Failed to parse search response: {0}")]
    Parse(String),
}

// --- Argument and Output Structs ---
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct WebSearchArgs {
    query: String,
}

// --- Tool Struct ---
#[derive(Clone)]
#[allow(dead_code)]
pub struct WebSearchTool {
    client: reqwest::Client,
    api_key: String,
}

impl WebSearchTool {
    #[allow(dead_code)]
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }
}

// --- API Response Structs (generic) ---
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct SearchResponse {
    web: Option<SearchResults>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct SearchResults {
    results: Vec<SearchResult>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct SearchResult {
    title: String,
    url: String,
    description: String,
}

// --- Google CSE Client and DTOs ---
#[allow(dead_code)]
const GOOGLE_CSE_ENDPOINT: &str = "https://www.googleapis.com/customsearch/v1";

#[derive(Clone)]
#[allow(dead_code)]
struct GoogleCseClient {
    client: reqwest::Client,
    api_key: String,
    cx: String,
}

#[allow(dead_code)]
impl GoogleCseClient {
    fn new(client: reqwest::Client, api_key: String, cx: String) -> Self {
        Self {
            client,
            api_key,
            cx,
        }
    }

    /// Build a request for a Google CSE query.
    /// - q: the search query
    /// - num: optional number of results (1..=10). If None, Google default applies.
    fn build_request(&self, q: &str, num: Option<u8>) -> reqwest::RequestBuilder {
        let mut req = self
            .client
            .get(GOOGLE_CSE_ENDPOINT)
            .query(&[
                ("key", self.api_key.as_str()),
                ("cx", self.cx.as_str()),
                ("q", q),
            ])
            .query(&[("safe", "off")]);

        if let Some(n) = num {
            let capped = n.min(10).max(1);
            req = req.query(&[("num", capped.to_string())]);
        }

        req
    }
}

// Google CSE JSON response DTOs (subset)
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct GoogleSearchResponse {
    items: Option<Vec<GoogleSearchItem>>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct GoogleSearchItem {
    title: String,
    link: String,
    snippet: String,
}

// --- Mappers ---
#[allow(dead_code)]
fn format_google_results(items: &[GoogleSearchItem], limit: usize) -> String {
    if items.is_empty() {
        return "No web results found.".to_string();
    }

    items
        .iter()
        .take(limit)
        .map(|it| {
            format!(
                "Title: {}\nURL: {}\nSnippet: {}\n",
                it.title, it.link, it.snippet
            )
        })
        .collect::<Vec<_>>()
        .join("\n---\n")
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
            description: "Searches the web for a given query. Use this for questions about current events, recent information, or topics that may have changed since the model's last training."
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
            let body = response.text().await.unwrap_or_default();
            let message = if body.is_empty() {
                "no response body".to_string()
            } else {
                let trimmed = body.trim();
                let max = 1000.min(trimmed.len());
                trimmed[..max].to_string()
            };
            return Err(WebSearchError::Api { status, message });
        }

        let search_response: SearchResponse = response
            .json()
            .await
            .map_err(|e| WebSearchError::Parse(e.to_string()))?;

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
