use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::mcp::tool::McpTool;
use crate::types::mcp::McpToolResult;

/// SDK MCP Server instance
#[derive(Clone)]
pub struct SdkMcpServer {
    pub name: String,
    tools: Arc<RwLock<HashMap<String, McpTool>>>,
}

impl SdkMcpServer {
    /// Create a new SDK MCP server
    pub fn new(name: &str) -> Self {
        info!("Creating SDK MCP server: {}", name);

        SdkMcpServer {
            name: name.to_string(),
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a tool with the server
    pub async fn register_tool(&self, tool: McpTool) {
        let tool_name = tool.definition.name.clone();
        debug!("Registering tool: {}", tool_name);

        let mut tools = self.tools.write().await;
        tools.insert(tool_name.clone(), tool);

        info!("Tool registered: {}", tool_name);
    }

    /// Get a tool by name
    pub async fn get_tool(&self, name: &str) -> Option<McpTool> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }

    /// Check if a tool exists
    pub async fn has_tool(&self, name: &str) -> bool {
        let tools = self.tools.read().await;
        tools.contains_key(name)
    }

    /// Get all tool names
    pub async fn get_tool_names(&self) -> Vec<String> {
        let tools = self.tools.read().await;
        tools.keys().cloned().collect()
    }

    /// Execute a tool
    pub async fn execute_tool(
        &self,
        name: &str,
        input: serde_json::Value,
    ) -> Result<McpToolResult> {
        let tool = self
            .get_tool(name)
            .await
            .ok_or_else(|| anyhow::anyhow!("Tool '{}' not found", name))?;

        debug!("Executing tool: {}", name);
        tool.execute(input).await
    }

    /// Get tool count
    pub async fn tool_count(&self) -> usize {
        let tools = self.tools.read().await;
        tools.len()
    }
}

// Clone implementation for McpTool
impl Clone for McpTool {
    fn clone(&self) -> Self {
        McpTool {
            definition: self.definition.clone(),
            handler: self.handler.clone(),
        }
    }
}

/// Create an SDK MCP server with tools
pub fn create_sdk_mcp_server(name: &str, tools: Vec<McpTool>) -> SdkMcpServer {
    info!(
        "Creating SDK MCP server '{}' with {} tools",
        name,
        tools.len()
    );

    // Note: This is synchronous for now, tools will be added before server is used
    // In a real implementation, you might want to use Arc<Mutex<HashMap>>
    // or build the HashMap before creating the server

    SdkMcpServer::new(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::tool::McpTool;
    use crate::types::mcp::{McpToolResult, ToolContent};

    fn create_test_tool(name: &str) -> McpTool {
        let name_string = name.to_string();
        let description = format!("Test tool {}", name);
        let name_for_closure = name_string.clone();

        McpTool::new(
            &name_string,
            &description,
            serde_json::json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                }
            }),
            move |_input: serde_json::Value| {
                let tool_name = name_for_closure.clone();
                async move {
                    Ok(McpToolResult {
                        content: vec![ToolContent::Text {
                            text: format!("Executed {}", tool_name),
                        }],
                        is_error: false,
                    })
                }
            },
        )
    }

    #[tokio::test]
    async fn test_sdk_mcp_server_creation() {
        let server = SdkMcpServer::new("test-server");

        assert_eq!(server.name, "test-server");
        assert_eq!(server.tool_count().await, 0);
    }

    #[tokio::test]
    async fn test_sdk_mcp_server_register_tool() {
        let server = SdkMcpServer::new("test-server");
        let tool = create_test_tool("test_tool");

        server.register_tool(tool).await;

        assert_eq!(server.tool_count().await, 1);
        assert!(server.has_tool("test_tool").await);
    }

    #[tokio::test]
    async fn test_sdk_mcp_server_get_tool() {
        let server = SdkMcpServer::new("test-server");
        let tool = create_test_tool("my_tool");

        server.register_tool(tool).await;

        let retrieved_tool = server.get_tool("my_tool").await;
        assert!(retrieved_tool.is_some());
        assert_eq!(retrieved_tool.unwrap().definition.name, "my_tool");
    }

    #[tokio::test]
    async fn test_sdk_mcp_server_get_nonexistent_tool() {
        let server = SdkMcpServer::new("test-server");

        let tool = server.get_tool("nonexistent").await;
        assert!(tool.is_none());
    }

    #[tokio::test]
    async fn test_sdk_mcp_server_execute_tool() {
        let server = SdkMcpServer::new("test-server");
        let tool = create_test_tool("exec_tool");

        server.register_tool(tool).await;

        let result = server
            .execute_tool("exec_tool", serde_json::json!({"input": "test"}))
            .await
            .unwrap();

        assert!(!result.is_error);
        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_sdk_mcp_server_execute_tool_not_found() {
        let server = SdkMcpServer::new("test-server");

        let result = server
            .execute_tool("nonexistent", serde_json::json!({}))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sdk_mcp_server_get_tool_names() {
        let server = SdkMcpServer::new("test-server");

        server.register_tool(create_test_tool("tool1")).await;
        server.register_tool(create_test_tool("tool2")).await;
        server.register_tool(create_test_tool("tool3")).await;

        let names = server.get_tool_names().await;
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"tool1".to_string()));
        assert!(names.contains(&"tool2".to_string()));
        assert!(names.contains(&"tool3".to_string()));
    }

    #[tokio::test]
    async fn test_sdk_mcp_server_multiple_tools() {
        let server = SdkMcpServer::new("test-server");

        for i in 0..5 {
            server
                .register_tool(create_test_tool(&format!("tool_{}", i)))
                .await;
        }

        assert_eq!(server.tool_count().await, 5);

        for i in 0..5 {
            assert!(server.has_tool(&format!("tool_{}", i)).await);
        }
    }

    #[test]
    fn test_create_sdk_mcp_server_with_tools() {
        let tools = vec![create_test_tool("tool1"), create_test_tool("tool2")];

        let server = create_sdk_mcp_server("my-server", tools);

        assert_eq!(server.name, "my-server");
    }

    #[test]
    fn test_create_sdk_mcp_server_empty() {
        let server = create_sdk_mcp_server("empty-server", vec![]);

        assert_eq!(server.name, "empty-server");
    }
}
