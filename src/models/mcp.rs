use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl McpResponse {
    /// Creates a successful MCP response
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Creates an error MCP response
    pub fn error(id: Value, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(McpError {
                code,
                message,
                data: None,
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    pub arguments: Value,
}

// RDedupe specific tool arguments
#[derive(Debug, Serialize, Deserialize)]
pub struct DedupeArgs {
    pub path: String,
    #[serde(default)]
    pub pattern: String,
    #[serde(default)]
    pub pattern_type: String,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub no_ignore: bool,
    #[serde(default)]
    pub max_depth: Option<usize>,
    #[serde(default)]
    pub similarity: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchArgs {
    pub path: String,
    #[serde(default)]
    pub pattern: String,
    #[serde(default)]
    pub pattern_type: String,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub no_ignore: bool,
    #[serde(default)]
    pub max_depth: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CountArgs {
    pub path: String,
    #[serde(default)]
    pub pattern: String,
    #[serde(default)]
    pub pattern_type: String,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub no_ignore: bool,
    #[serde(default)]
    pub max_depth: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutliersArgs {
    pub path: String,
    pub min_size: Option<String>,
    #[serde(default = "default_top_n")]
    pub top_n: usize,
    #[serde(default = "default_std_dev_threshold")]
    pub std_dev_threshold: f64,
    #[serde(default = "default_true")]
    pub check_hidden_consumers: bool,
    #[serde(default = "default_true")]
    pub check_patterns: bool,
}

fn default_top_n() -> usize {
    20
}

fn default_std_dev_threshold() -> f64 {
    2.0
}

fn default_true() -> bool {
    true
}
