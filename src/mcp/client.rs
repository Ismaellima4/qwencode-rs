use anyhow::Result;
use tracing::{debug, info};

use crate::mcp::server::SdkMcpServer;
use crate::types::mcp::McpToolResult;

/// MCP Client for interacting with MCP servers
pub struct McpClient {
    server: SdkMcpServer,
}

impl McpClient {
    /// Create a new MCP client for a server
    pub fn new(server: SdkMcpServer) -> Self {
        info!("Creating MCP client for server: {}", server.name);

        McpClient { server }
    }

    /// Call a tool on the server
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<McpToolResult> {
        debug!(
            "Calling tool '{}' with arguments: {:?}",
            tool_name, arguments
        );

        let result = self.server.execute_tool(tool_name, arguments).await?;

        debug!(
            "Tool '{}' returned {} content items",
            tool_name,
            result.content.len()
        );
        Ok(result)
    }

    /// List available tools
    pub async fn list_tools(&self) -> Vec<String> {
        self.server.get_tool_names().await
    }

    /// Check if a tool is available
    pub async fn has_tool(&self, tool_name: &str) -> bool {
        self.server.has_tool(tool_name).await
    }

    /// Get the server name
    pub fn server_name(&self) -> &str {
        &self.server.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::tool::McpTool;
    use crate::types::mcp::{McpToolResult, ToolContent};

    fn create_test_server() -> SdkMcpServer {
        let _tool = McpTool::new(
            "test_tool",
            "Test tool",
            serde_json::json!({}),
            |_input: serde_json::Value| async move {
                Ok(McpToolResult {
                    content: vec![ToolContent::Text {
                        text: "test result".to_string(),
                    }],
                    is_error: false,
                })
            },
        );

        // Note: In real implementation, we'd add tools properly
        // For now, testing client creation
        SdkMcpServer::new("test-server")
    }

    #[test]
    fn test_mcp_client_creation() {
        let server = create_test_server();
        let client = McpClient::new(server);

        assert_eq!(client.server_name(), "test-server");
    }

    #[tokio::test]
    async fn test_mcp_client_list_tools() {
        let server = create_test_server();
        let client = McpClient::new(server);

        let tools = client.list_tools().await;
        assert_eq!(tools.len(), 0); // No tools added
    }

    #[tokio::test]
    async fn test_mcp_client_has_tool_check() {
        let server = create_test_server();
        let client = McpClient::new(server);

        let has_tool = client.has_tool("nonexistent").await;
        assert!(!has_tool);
    }
}
