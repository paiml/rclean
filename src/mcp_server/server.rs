use crate::models::mcp::{McpRequest, McpResponse};
use anyhow::Result;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tracing::{debug, error, info};

pub struct McpServer;

impl McpServer {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self) -> Result<()> {
        info!("Starting RDedupe MCP server");

        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let mut reader = AsyncBufReader::new(stdin);
        let mut writer = stdout;

        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await?;

            if bytes_read == 0 {
                debug!("EOF received, shutting down");
                break;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            debug!("Received request: {}", trimmed);

            match serde_json::from_str::<McpRequest>(trimmed) {
                Ok(request) => {
                    let response = self.handle_request(request).await;
                    let response_str = serde_json::to_string(&response)?;

                    debug!("Sending response: {}", response_str);
                    writer.write_all(response_str.as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                    writer.flush().await?;
                },
                Err(e) => {
                    error!("Failed to parse request: {}", e);
                    let error_response =
                        McpResponse::error(Value::Null, -32700, "Parse error".to_string());
                    let response_str = serde_json::to_string(&error_response)?;
                    writer.write_all(response_str.as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                    writer.flush().await?;
                },
            }
        }

        Ok(())
    }

    async fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            "initialize" => super::handlers::handle_initialize(request),
            "tools/list" => super::handlers::handle_tools_list(request),
            "tools/call" => super::handlers::handle_tool_call(request).await,
            _ => McpResponse::error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::mcp::McpRequest;
    use serde_json::json;

    #[test]
    fn test_mcp_server_new() {
        let _server = McpServer::new();
        // Just ensure it can be created
        let _default_server = McpServer;
    }

    #[tokio::test]
    async fn test_handle_request_initialize() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "initialize".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_handle_request_tools_list() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_handle_request_unknown_method() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "unknown/method".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert!(error.message.contains("Method not found"));
    }
}
