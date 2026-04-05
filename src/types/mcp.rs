use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP server transport type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub enum McpTransport {
    /// stdio transport
    Stdio {
        command: String,
        args: Vec<String>,
        #[serde(default)]
        env: Option<HashMap<String, String>>,
    },
    /// Server-Sent Events transport
    Sse { url: String },
    /// HTTP transport
    Http {
        url: String,
        #[serde(default)]
        headers: Option<HashMap<String, String>>,
    },
    /// SDK embedded server
    Sdk {
        #[serde(skip)]
        instance: Option<()>, // Placeholder for SDK server instance
    },
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub transport: McpTransport,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub tools: Option<Vec<McpToolDefinition>>,
}

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// MCP tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    pub content: Vec<ToolContent>,
    #[serde(default)]
    pub is_error: bool,
}

/// Tool content types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolContent {
    Text {
        text: String,
    },
    Image {
        data: String,
        mime_type: String,
    },
    Resource {
        uri: String,
        mime_type: String,
        text: Option<String>,
        blob: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_transport_stdio() {
        let transport = McpTransport::Stdio {
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            env: Some(HashMap::from([("PORT".to_string(), "3000".to_string())])),
        };

        match &transport {
            McpTransport::Stdio { command, args, env } => {
                assert_eq!(command, "node");
                assert_eq!(args, &vec!["server.js"]);
                assert!(env.is_some());
            }
            _ => panic!("Expected Stdio variant"),
        }
    }

    #[test]
    fn test_mcp_transport_sse() {
        let transport = McpTransport::Sse {
            url: "http://localhost:3000/sse".to_string(),
        };

        match &transport {
            McpTransport::Sse { url } => {
                assert_eq!(url, "http://localhost:3000/sse");
            }
            _ => panic!("Expected Sse variant"),
        }
    }

    #[test]
    fn test_mcp_transport_http() {
        let transport = McpTransport::Http {
            url: "http://localhost:3000/mcp".to_string(),
            headers: Some(HashMap::from([(
                "Authorization".to_string(),
                "Bearer token".to_string(),
            )])),
        };

        match &transport {
            McpTransport::Http { url, headers } => {
                assert_eq!(url, "http://localhost:3000/mcp");
                assert!(headers.is_some());
            }
            _ => panic!("Expected Http variant"),
        }
    }

    #[test]
    fn test_mcp_server_config_creation() {
        let config = McpServerConfig {
            name: "test-server".to_string(),
            transport: McpTransport::Stdio {
                command: "python".to_string(),
                args: vec!["mcp_server.py".to_string()],
                env: None,
            },
            timeout_ms: Some(30000),
            tools: None,
        };

        assert_eq!(config.name, "test-server");
        assert_eq!(config.timeout_ms, Some(30000));
    }

    #[test]
    fn test_mcp_tool_definition() {
        let tool = McpToolDefinition {
            name: "calculate".to_string(),
            description: "Perform calculation".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "expression": {"type": "string"}
                }
            }),
        };

        assert_eq!(tool.name, "calculate");
        assert_eq!(tool.description, "Perform calculation");
        assert!(tool.input_schema.is_object());
    }

    #[test]
    fn test_mcp_tool_result() {
        let result = McpToolResult {
            content: vec![ToolContent::Text {
                text: "42".to_string(),
            }],
            is_error: false,
        };

        assert_eq!(result.content.len(), 1);
        assert!(!result.is_error);

        match &result.content[0] {
            ToolContent::Text { text } => assert_eq!(text, "42"),
            _ => panic!("Expected Text content"),
        }
    }

    #[test]
    fn test_tool_content_text() {
        let content = ToolContent::Text {
            text: "Hello world".to_string(),
        };

        let serialized = serde_json::to_string(&content).unwrap();
        assert!(serialized.contains("\"type\":\"text\""));
        assert!(serialized.contains("\"text\":\"Hello world\""));
    }

    #[test]
    fn test_tool_content_image() {
        let content = ToolContent::Image {
            data: "base64data".to_string(),
            mime_type: "image/png".to_string(),
        };

        let serialized = serde_json::to_string(&content).unwrap();
        assert!(serialized.contains("\"type\":\"image\""));
        assert!(serialized.contains("\"mime_type\":\"image/png\""));
    }

    #[test]
    fn test_tool_content_resource() {
        let content = ToolContent::Resource {
            uri: "file:///test.txt".to_string(),
            mime_type: "text/plain".to_string(),
            text: Some("content".to_string()),
            blob: None,
        };

        let serialized = serde_json::to_string(&content).unwrap();
        assert!(serialized.contains("\"type\":\"resource\""));
        assert!(serialized.contains("\"uri\":\"file:///test.txt\""));
    }

    #[test]
    fn test_mcp_tool_result_with_error() {
        let result = McpToolResult {
            content: vec![ToolContent::Text {
                text: "Error occurred".to_string(),
            }],
            is_error: true,
        };

        assert!(result.is_error);
    }

    #[test]
    fn test_mcp_tool_result_multiple_contents() {
        let result = McpToolResult {
            content: vec![
                ToolContent::Text {
                    text: "Result".to_string(),
                },
                ToolContent::Image {
                    data: "img_data".to_string(),
                    mime_type: "image/jpeg".to_string(),
                },
            ],
            is_error: false,
        };

        assert_eq!(result.content.len(), 2);
        assert!(matches!(&result.content[0], ToolContent::Text { .. }));
        assert!(matches!(&result.content[1], ToolContent::Image { .. }));
    }

    #[test]
    fn test_stdio_transport_without_env() {
        let transport = McpTransport::Stdio {
            command: "bash".to_string(),
            args: vec!["script.sh".to_string()],
            env: None,
        };

        match &transport {
            McpTransport::Stdio { env, .. } => {
                assert!(env.is_none());
            }
            _ => panic!("Expected Stdio variant"),
        }
    }

    #[test]
    fn test_http_transport_without_headers() {
        let transport = McpTransport::Http {
            url: "http://localhost:8080".to_string(),
            headers: None,
        };

        match &transport {
            McpTransport::Http { headers, .. } => {
                assert!(headers.is_none());
            }
            _ => panic!("Expected Http variant"),
        }
    }
}
