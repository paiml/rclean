//! MCP integration tests for rclean

use rclean::models::mcp::{McpRequest, McpResponse};
use serde_json::json;

#[test]
fn test_mcp_response_success() {
    let response = McpResponse::success(json!(1), json!({"result": "ok"}));

    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id, json!(1));
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_mcp_response_error() {
    let response = McpResponse::error(json!(1), -32601, "Method not found".to_string());

    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id, json!(1));
    assert!(response.result.is_none());
    assert!(response.error.is_some());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32601);
    assert_eq!(error.message, "Method not found");
}

#[test]
fn test_mcp_handlers() {
    use rclean::mcp_server::handlers;

    // Test initialize
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "initialize".to_string(),
        params: None,
    };

    let response = handlers::handle_initialize(request);
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    assert!(result["protocolVersion"].is_string());
    assert!(result["capabilities"].is_object());
    assert!(result["serverInfo"]["name"].as_str().unwrap() == "rclean");
}

#[test]
fn test_tools_list() {
    use rclean::mcp_server::handlers;

    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/list".to_string(),
        params: None,
    };

    let response = handlers::handle_tools_list(request);
    assert!(response.result.is_some());

    let result = response.result.unwrap();
    let tools = result["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 5);

    // Check tool names
    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(tool_names.contains(&"dedupe"));
    assert!(tool_names.contains(&"search"));
    assert!(tool_names.contains(&"count"));
    assert!(tool_names.contains(&"outliers"));
    assert!(tool_names.contains(&"analyze_file_clusters"));
}
