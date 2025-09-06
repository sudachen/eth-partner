//! Tests for the tool definition generator.

use mcp_wallet::commands::tool_definition::generate_tool_definition;
use serde_json::Value;

#[test]
fn test_generate_tool_definition_structure() {
    let definition = generate_tool_definition();
    let json = serde_json::to_value(&definition).expect("Failed to serialize tool definition");

    assert!(json.is_object());
    let tool = &json["tool"];
    assert!(tool.is_object());
    assert_eq!(tool["name"], "eth_wallet_manager");
    assert!(tool["description"].is_string());

    let functions = &tool["functions"];
    assert!(functions.is_array());
    assert!(functions.as_array().unwrap().len() >= 5, "Should have at least 5 functions");

    // Check the structure of the first function (new-account)
    let new_account_fn = &functions[0];
    assert_eq!(new_account_fn["name"], "new-account");
    assert!(new_account_fn["description"].is_string());

    let params = &new_account_fn["parameters"];
    assert!(params.is_object());
    assert_eq!(params["type"], "object");

    let properties = &params["properties"];
    assert!(properties.is_object());
    assert!(properties["alias"].is_object());
    assert_eq!(properties["alias"]["type"], "string");
}

#[test]
fn test_get_tool_definition_cli_flag() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_mcp-wallet"))
        .arg("--get-tool-definition")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("Output is not valid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("Output is not valid JSON");

    assert_eq!(json["tool"]["name"], "eth_wallet_manager");
}
