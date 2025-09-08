use httpmock::prelude::*;
use repl::tools::web_search::{WebSearchArgs, WebSearchError, WebSearchTool};
use rig::tool::Tool;

#[tokio::test]
async fn test_google_cse_parsing_success() {
    let server = MockServer::start();

    let body = serde_json::json!({
        "items": [
            { "title": "Result One", "link": "https://example.com/one", "snippet": "Snippet one" },
            { "title": "Result Two", "link": "https://example.com/two", "snippet": "Snippet two" }
        ]
    });

    let _m = server.mock(|when, then| {
        when.method(GET)
            .path("/")
            .query_param_exists("key")
            .query_param_exists("cx")
            .query_param_exists("q");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(body);
    });

    let tool = WebSearchTool::new_with_endpoint(
        "test-api-key".to_string(),
        "test-cx".to_string(),
        server.base_url(),
    );

    let out = tool
        .call(WebSearchArgs {
            query: "test".to_string(),
            num: Some(2),
        })
        .await
        .expect("tool call should succeed");

    let v: serde_json::Value = serde_json::from_str(&out).expect("valid json output");
    assert_eq!(v["provider"], "google_cse");
    let total = v["total"].as_u64().unwrap_or(0);
    let results = v["results"].as_array().cloned().unwrap_or_default();
    assert_eq!(total, 2);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["index"], 1);
    assert_eq!(results[0]["title"], "Result One");
    assert_eq!(results[0]["url"], "https://example.com/one");
    assert_eq!(results[0]["snippet"], "Snippet one");
}

#[tokio::test]
async fn test_google_cse_empty_items_returns_empty_results() {
    let server = MockServer::start();

    let body = serde_json::json!({ "items": [] });

    let _m = server.mock(|when, then| {
        when.method(GET)
            .path("/")
            .query_param_exists("key")
            .query_param_exists("cx")
            .query_param_exists("q");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(body);
    });

    let tool = WebSearchTool::new_with_endpoint(
        "test-api-key".to_string(),
        "test-cx".to_string(),
        server.base_url(),
    );

    let out = tool
        .call(WebSearchArgs {
            query: "noresults".to_string(),
            num: Some(5),
        })
        .await
        .expect("tool call should succeed");

    let v: serde_json::Value = serde_json::from_str(&out).expect("valid json output");
    assert_eq!(v["provider"], "google_cse");
    let total = v["total"].as_u64().unwrap_or(1);
    let results = v["results"]
        .as_array()
        .cloned()
        .unwrap_or_else(|| vec![serde_json::json!("not empty")]);
    assert_eq!(total, 0);
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_google_cse_5xx_returns_readable_error() {
    let server = MockServer::start();

    let _m = server.mock(|when, then| {
        when.method(GET)
            .path("/")
            .query_param_exists("key")
            .query_param_exists("cx")
            .query_param_exists("q");
        then.status(500)
            .header("content-type", "application/json")
            .body("{\"error\":\"internal\"}");
    });

    let tool = WebSearchTool::new_with_endpoint(
        "test-api-key".to_string(),
        "test-cx".to_string(),
        server.base_url(),
    );

    let err = tool
        .call(WebSearchArgs {
            query: "trigger error".to_string(),
            num: None,
        })
        .await
        .expect_err("expected an error for 500 response");

    match err {
        WebSearchError::Api { status, message } => {
            assert_eq!(status, 500);
            assert!(message.contains("internal") || !message.is_empty());
        }
        other => panic!("unexpected error variant: {:?}", other),
    }
}
