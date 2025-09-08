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
    pub query: String,
    #[serde(default)]
    pub num: Option<u8>,
}

// --- Tool Struct ---
#[derive(Clone)]
#[allow(dead_code)]
pub struct WebSearchTool {
    google: GoogleCseClient,
    default_num: u8,
}

impl WebSearchTool {
    #[allow(dead_code)]
    pub fn new(api_key: String, engine_id: String) -> Self {
        let client = reqwest::Client::new();
        let google = GoogleCseClient::new(client, api_key, engine_id);
        Self {
            google,
            default_num: 5,
        }
    }

    /// Public helper to construct a tool with a custom base URL (primarily for tests).
    pub fn new_with_endpoint(api_key: String, engine_id: String, base_url: String) -> Self {
        let client = reqwest::Client::new();
        let google = GoogleCseClient::with_base_url(client, api_key, engine_id, base_url);
        Self {
            google,
            default_num: 5,
        }
    }
}

// (Removed Brave/generic response structs; using Google DTOs below)

#[derive(Clone)]
#[allow(dead_code)]
struct GoogleCseClient {
    client: reqwest::Client,
    api_key: String,
    cx: String,
    base_url: String,
}

#[allow(dead_code)]
impl GoogleCseClient {
    fn new(client: reqwest::Client, api_key: String, cx: String) -> Self {
        Self {
            client,
            api_key,
            cx,
            base_url: "https://www.googleapis.com/customsearch/v1".to_string(),
        }
    }

    fn with_base_url(
        client: reqwest::Client,
        api_key: String,
        cx: String,
        base_url: String,
    ) -> Self {
        Self {
            client,
            api_key,
            cx,
            base_url,
        }
    }

    /// Build a request for a Google CSE query.
    /// - q: the search query
    /// - num: optional number of results (1..=10). If None, Google default applies.
    fn build_request(&self, q: &str, num: Option<u8>) -> reqwest::RequestBuilder {
        let mut req = self
            .client
            .get(&self.base_url)
            .query(&[
                ("key", self.api_key.as_str()),
                ("cx", self.cx.as_str()),
                ("q", q),
            ])
            .query(&[("safe", "off")]);

        if let Some(n) = num {
            let capped = n.clamp(1, 10);
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
    title: Option<String>,
    link: Option<String>,
    snippet: Option<String>,
}

// --- Mappers ---
#[allow(dead_code)]
fn format_google_results(items: &[GoogleSearchItem], limit: usize) -> String {
    if items.is_empty() {
        return "No web results found.".to_string();
    }

    let mut formatted: Vec<String> = Vec::new();
    for it in items.iter() {
        if let (Some(title), Some(link)) = (&it.title, &it.link) {
            let snippet = it.snippet.as_deref().unwrap_or("");
            formatted.push(format!(
                "Title: {}\nURL: {}\nSnippet: {}\n",
                title, link, snippet
            ));
        }
        if formatted.len() >= limit {
            break;
        }
    }

    if formatted.is_empty() {
        "No web results found.".to_string()
    } else {
        formatted.join("\n---\n")
    }
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
            description: "Searches the web for a given query and returns JSON with { total, results:[{index,title,url,snippet}], provider }. Use for up-to-date info."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query."
                    },
                    "num": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 10,
                        "description": "Optional number of results to return (1..10). Defaults to 5."
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let requested = args.num.unwrap_or(self.default_num);
        // Log query length and requested num (avoid logging full query content)
        println!(
            "web_search: query_len={}, requested_num={}",
            args.query.len(),
            requested
        );
        let response = self
            .google
            .build_request(&args.query, Some(requested))
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

        let search_response: GoogleSearchResponse = response
            .json()
            .await
            .map_err(|e| WebSearchError::Parse(e.to_string()))?;

        let items = search_response.items.unwrap_or_default();

        // Build JSON output expected by the agent
        let mut results = Vec::new();
        let mut idx: usize = 1;
        for it in items.iter() {
            if let (Some(title), Some(link)) = (&it.title, &it.link) {
                let snippet = it.snippet.clone().unwrap_or_default();
                results.push(json!({
                    "index": idx,
                    "title": title,
                    "url": link,
                    "snippet": snippet,
                }));
                idx += 1;
                if idx > self.default_num as usize {
                    break;
                }
            }
        }

        println!("web_search: results_count={}", results.len());

        let out = json!({
            "total": results.len(),
            "results": results,
            "provider": "google_cse"
        });

        Ok(out.to_string())
    }
}
