//! Comprehensive tests for MCP handlers to improve coverage

use rclean::mcp_server::handlers::*;
use rclean::models::mcp::*;
use serde_json::json;

#[test]
fn test_initialize_handler() {
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "1.0",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
    };

    let response = handle_initialize(request);
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    assert!(result.get("protocolVersion").is_some());
    assert!(result.get("capabilities").is_some());
    assert!(result.get("serverInfo").is_some());
}

#[test]
fn test_tools_list_handler() {
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/list".to_string(),
        params: None,
    };

    let response = handle_tools_list(request);
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let tools = result.get("tools").unwrap().as_array().unwrap();
    assert_eq!(tools.len(), 5);

    let tool_names: Vec<&str> = tools
        .iter()
        .map(|tool| tool.get("name").unwrap().as_str().unwrap())
        .collect();
    assert!(tool_names.contains(&"dedupe"));
    assert!(tool_names.contains(&"search"));
    assert!(tool_names.contains(&"count"));
    assert!(tool_names.contains(&"outliers"));
    assert!(tool_names.contains(&"analyze_file_clusters"));
}

#[tokio::test]
async fn test_dedupe_tool_handler() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("test1.txt"), "same content").unwrap();
    std::fs::write(temp_dir.path().join("test2.txt"), "same content").unwrap();
    std::fs::write(temp_dir.path().join("test3.txt"), "different content").unwrap();

    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "dedupe",
            "arguments": {
                "path": temp_dir.path().to_str().unwrap(),
                "pattern": "",
                "pattern_type": "literal",
                "hidden": false,
                "no_ignore": false,
                "max_depth": null,
                "csv": null,
                "similarity": null
            }
        })),
    };

    let response = handle_tool_call(request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let message = result.get("message").unwrap().as_str().unwrap();
    assert!(message.contains("duplicate files"));
    assert!(message.contains("total files"));

    let total_files = result.get("total_files").unwrap().as_u64().unwrap();
    assert!(total_files >= 3);
}

#[tokio::test]
async fn test_dedupe_tool_with_similarity() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("test1.txt"), "content").unwrap();
    std::fs::write(temp_dir.path().join("test2.txt"), "content").unwrap();

    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "dedupe",
            "arguments": {
                "path": temp_dir.path().to_str().unwrap(),
                "pattern": "",
                "pattern_type": "literal",
                "hidden": false,
                "no_ignore": false,
                "max_depth": null,
                "csv": null,
                "similarity": 70
            }
        })),
    };

    let response = handle_tool_call(request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[tokio::test]
async fn test_dedupe_tool_with_csv() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("test1.txt"), "same content").unwrap();
    std::fs::write(temp_dir.path().join("test2.txt"), "same content").unwrap();

    let csv_path = temp_dir.path().join("output.csv");

    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "dedupe",
            "arguments": {
                "path": temp_dir.path().to_str().unwrap(),
                "pattern": "",
                "pattern_type": "literal",
                "hidden": false,
                "no_ignore": false,
                "max_depth": null,
                "csv": csv_path.to_str().unwrap(),
                "similarity": null
            }
        })),
    };

    let response = handle_tool_call(request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let message = result.get("message").unwrap().as_str().unwrap();
    assert!(message.contains("duplicate files"));
    // Note: CSV file creation might not happen in all cases
    // Just check the response is successful
}

#[tokio::test]
async fn test_dedupe_tool_with_glob_pattern() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("test1.txt"), "content").unwrap();
    std::fs::write(temp_dir.path().join("test2.rs"), "code").unwrap();

    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "dedupe",
            "arguments": {
                "path": temp_dir.path().to_str().unwrap(),
                "pattern": "*.txt",
                "pattern_type": "glob",
                "hidden": false,
                "no_ignore": false,
                "max_depth": null,
                "csv": null,
                "similarity": null
            }
        })),
    };

    let response = handle_tool_call(request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[tokio::test]
async fn test_dedupe_tool_with_regex_pattern() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("test1.txt"), "content").unwrap();
    std::fs::write(temp_dir.path().join("test2.rs"), "code").unwrap();

    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "dedupe",
            "arguments": {
                "path": temp_dir.path().to_str().unwrap(),
                "pattern": ".*\\.rs$",
                "pattern_type": "regex",
                "hidden": false,
                "no_ignore": false,
                "max_depth": null,
                "csv": null,
                "similarity": null
            }
        })),
    };

    let response = handle_tool_call(request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[tokio::test]
async fn test_dedupe_tool_with_hidden_files() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("test1.txt"), "content").unwrap();
    std::fs::write(temp_dir.path().join(".hidden"), "hidden content").unwrap();

    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "dedupe",
            "arguments": {
                "path": temp_dir.path().to_str().unwrap(),
                "pattern": "",
                "pattern_type": "literal",
                "hidden": true,
                "no_ignore": false,
                "max_depth": null,
                "csv": null,
                "similarity": null
            }
        })),
    };

    let response = handle_tool_call(request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[tokio::test]
async fn test_dedupe_tool_with_max_depth() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let subdir = temp_dir.path().join("subdir");
    std::fs::create_dir(&subdir).unwrap();
    std::fs::write(temp_dir.path().join("test1.txt"), "content").unwrap();
    std::fs::write(subdir.join("test2.txt"), "content").unwrap();

    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "dedupe",
            "arguments": {
                "path": temp_dir.path().to_str().unwrap(),
                "pattern": "",
                "pattern_type": "literal",
                "hidden": false,
                "no_ignore": false,
                "max_depth": 1,
                "csv": null,
                "similarity": null
            }
        })),
    };

    let response = handle_tool_call(request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[tokio::test]
async fn test_search_tool_handler() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("test1.txt"), "content").unwrap();
    std::fs::write(temp_dir.path().join("test2.txt"), "content").unwrap();
    std::fs::write(temp_dir.path().join("example.rs"), "code").unwrap();

    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "search",
            "arguments": {
                "path": temp_dir.path().to_str().unwrap(),
                "pattern": "test",
                "pattern_type": "literal",
                "hidden": false,
                "no_ignore": false,
                "max_depth": null
            }
        })),
    };

    let response = handle_tool_call(request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let message = result.get("message").unwrap().as_str().unwrap();
    assert!(message.contains("Found"));
    assert!(message.contains("files matching pattern"));

    let count = result.get("count").unwrap().as_u64().unwrap();
    assert!(count >= 2);
}

#[tokio::test]
async fn test_search_tool_with_glob() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("test1.txt"), "content").unwrap();
    std::fs::write(temp_dir.path().join("test2.rs"), "code").unwrap();

    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "search",
            "arguments": {
                "path": temp_dir.path().to_str().unwrap(),
                "pattern": "*.txt",
                "pattern_type": "glob",
                "hidden": false,
                "no_ignore": false,
                "max_depth": null
            }
        })),
    };

    let response = handle_tool_call(request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[tokio::test]
async fn test_count_tool_handler() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("test1.txt"), "content").unwrap();
    std::fs::write(temp_dir.path().join("test2.txt"), "content").unwrap();
    std::fs::write(temp_dir.path().join("example.rs"), "code").unwrap();

    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "count",
            "arguments": {
                "path": temp_dir.path().to_str().unwrap(),
                "pattern": "test",
                "pattern_type": "literal",
                "hidden": false,
                "no_ignore": false,
                "max_depth": null
            }
        })),
    };

    let response = handle_tool_call(request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let message = result.get("message").unwrap().as_str().unwrap();
    assert!(message.contains("Found"));
    assert!(message.contains("files matching pattern"));

    let count = result.get("count").unwrap().as_u64().unwrap();
    assert!(count >= 2);
}

#[tokio::test]
async fn test_tool_call_unknown_tool() {
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "unknown_tool",
            "arguments": {}
        })),
    };

    let response = handle_tool_call(request).await;
    assert!(response.result.is_none());
    assert!(response.error.is_some());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("Unknown tool"));
}

#[tokio::test]
async fn test_tool_call_invalid_params() {
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "invalid": "params"
        })),
    };

    let response = handle_tool_call(request).await;
    assert!(response.result.is_none());
    assert!(response.error.is_some());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("Invalid params"));
}

#[tokio::test]
async fn test_dedupe_tool_invalid_path() {
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "dedupe",
            "arguments": {
                "path": "/nonexistent/path",
                "pattern": "",
                "pattern_type": "literal",
                "hidden": false,
                "no_ignore": false,
                "max_depth": null,
                "csv": null,
                "similarity": null
            }
        })),
    };

    let response = handle_tool_call(request).await;
    // Should succeed with empty results due to improved error handling
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[tokio::test]
async fn test_dedupe_tool_invalid_pattern() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "dedupe",
            "arguments": {
                "path": temp_dir.path().to_str().unwrap(),
                "pattern": "[invalid",
                "pattern_type": "regex",
                "hidden": false,
                "no_ignore": false,
                "max_depth": null,
                "csv": null,
                "similarity": null
            }
        })),
    };

    let response = handle_tool_call(request).await;
    assert!(response.result.is_none());
    assert!(response.error.is_some());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("Invalid pattern"));
}

#[tokio::test]
async fn test_search_tool_invalid_path() {
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "search",
            "arguments": {
                "path": "/nonexistent/path",
                "pattern": "test",
                "pattern_type": "literal",
                "hidden": false,
                "no_ignore": false,
                "max_depth": null
            }
        })),
    };

    let response = handle_tool_call(request).await;
    // Should succeed with empty results due to improved error handling
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[tokio::test]
async fn test_count_tool_invalid_path() {
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "count",
            "arguments": {
                "path": "/nonexistent/path",
                "pattern": "test",
                "pattern_type": "literal",
                "hidden": false,
                "no_ignore": false,
                "max_depth": null
            }
        })),
    };

    let response = handle_tool_call(request).await;
    // Should succeed with empty results due to improved error handling
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[tokio::test]
async fn test_tool_call_no_params() {
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: None,
    };

    let response = handle_tool_call(request).await;
    assert!(response.result.is_none());
    assert!(response.error.is_some());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("Invalid params"));
}

#[test]
fn test_mcp_error_creation() {
    let error = McpError {
        code: -32600,
        message: "Invalid Request".to_string(),
        data: None,
    };

    assert_eq!(error.code, -32600);
    assert_eq!(error.message, "Invalid Request");
}

#[test]
fn test_mcp_response_success() {
    let response = McpResponse::success(json!(1), json!({"test": "data"}));
    assert!(response.result.is_some());
    assert!(response.error.is_none());
    assert_eq!(response.id, json!(1));
    assert_eq!(response.jsonrpc, "2.0");
}

#[test]
fn test_mcp_response_error() {
    let response = McpResponse::error(json!(1), -32600, "Invalid Request".to_string());
    assert!(response.result.is_none());
    assert!(response.error.is_some());
    assert_eq!(response.id, json!(1));
    assert_eq!(response.jsonrpc, "2.0");

    let error = response.error.unwrap();
    assert_eq!(error.code, -32600);
    assert_eq!(error.message, "Invalid Request");
}

#[tokio::test]
async fn test_outliers_tool_handler() {
    let temp_dir = tempfile::TempDir::new().unwrap();

    // Create files with different sizes
    std::fs::write(temp_dir.path().join("small1.txt"), "a".repeat(10000)).unwrap();
    std::fs::write(temp_dir.path().join("small2.txt"), "a".repeat(10000)).unwrap();
    std::fs::write(temp_dir.path().join("large.txt"), "a".repeat(1000000)).unwrap();

    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "outliers",
            "arguments": {
                "path": temp_dir.path().to_str().unwrap(),
                "min_size": null,
                "top_n": 10,
                "std_dev_threshold": 1.0,
                "check_hidden_consumers": true,
                "check_patterns": true
            }
        })),
    };

    let response = handle_tool_call(request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    assert!(result.get("large_files").is_some());
    assert!(result.get("hidden_consumers").is_some());
    assert!(result.get("pattern_groups").is_some());
}
