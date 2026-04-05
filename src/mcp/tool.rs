use anyhow::Result;
use schemars::{schema_for, JsonSchema};
use serde::Deserialize;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::types::mcp::{McpToolDefinition, McpToolResult};

/// Tool handler function type
pub type ToolHandler = Arc<
    dyn Fn(serde_json::Value) -> Pin<Box<dyn Future<Output = Result<McpToolResult>> + Send>>
        + Send
        + Sync,
>;

/// MCP Tool with name, description, schema, and handler
pub struct McpTool {
    pub definition: McpToolDefinition,
    pub handler: ToolHandler,
}

impl McpTool {
    /// Create a new MCP tool
    pub fn new<F, Fut>(
        name: &str,
        description: &str,
        input_schema: serde_json::Value,
        handler: F,
    ) -> Self
    where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<McpToolResult>> + Send + 'static,
    {
        McpTool {
            definition: McpToolDefinition {
                name: name.to_string(),
                description: description.to_string(),
                input_schema,
            },
            handler: Arc::new(move |input| Box::pin(handler(input))),
        }
    }

    /// Execute the tool with given input
    pub async fn execute(&self, input: serde_json::Value) -> Result<McpToolResult> {
        (self.handler)(input).await
    }
}

/// Macro to create a tool with automatic schema generation
///
/// # Example
/// ```ignore
/// use qwencode_rs::tool;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, JsonSchema)]
/// struct AddArgs {
///     a: i32,
///     b: i32,
/// }
///
/// let tool = tool!(
///     "add",
///     "Add two numbers",
///     AddArgs,
///     |args: AddArgs| async move {
///         Ok(McpToolResult {
///             content: vec![ToolContent::Text {
///                 text: format!("{}", args.a + args.b),
///             }],
///             is_error: false,
///         })
///     }
/// );
/// ```
#[macro_export]
macro_rules! tool {
    ($name:expr, $description:expr, $args_type:ty, $handler:expr) => {{
        use schemars::schema_for;
        use $crate::mcp::tool::McpTool;
        use $crate::types::mcp::McpToolResult;
        use $crate::types::mcp::ToolContent;

        let schema = schemars::schema_for!($args_type);
        let schema_json = serde_json::to_value(schema).unwrap();

        McpTool::new(
            $name,
            $description,
            schema_json,
            move |input: serde_json::Value| {
                let handler = $handler;
                async move {
                    let args: $args_type = serde_json::from_value(input)?;
                    handler(args).await
                }
            },
        )
    }};
}

/// Helper function to create a tool without the macro (for dynamic tool creation)
pub fn create_tool<F, Fut, Args>(name: &str, description: &str, handler: F) -> McpTool
where
    F: Fn(Args) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<McpToolResult>> + Send + 'static,
    Args: for<'de> Deserialize<'de> + JsonSchema + 'static,
{
    let schema = schema_for!(Args);
    let schema_json = serde_json::to_value(schema).expect("Failed to serialize schema");
    let handler = Arc::new(handler);

    McpTool::new(
        name,
        description,
        schema_json,
        move |input: serde_json::Value| {
            let handler = handler.clone();
            async move {
                let args: Args = serde_json::from_value(input)?;
                handler(args).await
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::mcp::{McpToolResult, ToolContent};
    use serde::Deserialize;

    #[derive(Debug, Deserialize, JsonSchema)]
    #[allow(dead_code)]
    struct TestArgs {
        value: i32,
    }

    #[tokio::test]
    async fn test_mcp_tool_creation() {
        let tool = McpTool::new(
            "test_tool",
            "A test tool",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "value": {"type": "integer"}
                }
            }),
            |input: serde_json::Value| async move {
                let value = input["value"].as_i64().unwrap_or(0);
                Ok(McpToolResult {
                    content: vec![ToolContent::Text {
                        text: format!("Got value: {}", value),
                    }],
                    is_error: false,
                })
            },
        );

        assert_eq!(tool.definition.name, "test_tool");
        assert_eq!(tool.definition.description, "A test tool");
    }

    #[tokio::test]
    async fn test_mcp_tool_execution() {
        let tool = McpTool::new(
            "test_tool",
            "A test tool",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "value": {"type": "integer"}
                }
            }),
            |input: serde_json::Value| async move {
                let value = input["value"].as_i64().unwrap_or(0);
                Ok(McpToolResult {
                    content: vec![ToolContent::Text {
                        text: format!("Result: {}", value * 2),
                    }],
                    is_error: false,
                })
            },
        );

        let result = tool
            .execute(serde_json::json!({"value": 21}))
            .await
            .unwrap();

        assert!(!result.is_error);
        assert_eq!(result.content.len(), 1);

        match &result.content[0] {
            ToolContent::Text { text } => assert_eq!(text, "Result: 42"),
            _ => panic!("Expected Text content"),
        }
    }

    #[test]
    fn test_tool_definition_structure() {
        let tool = McpTool::new(
            "calc",
            "Calculate",
            serde_json::json!({"type": "object"}),
            |_input: serde_json::Value| async move {
                Ok(McpToolResult {
                    content: vec![],
                    is_error: false,
                })
            },
        );

        assert_eq!(tool.definition.name, "calc");
        assert_eq!(tool.definition.description, "Calculate");
        assert!(tool.definition.input_schema.is_object());
    }

    #[tokio::test]
    async fn test_tool_with_error() {
        let tool = McpTool::new(
            "failing_tool",
            "Always fails",
            serde_json::json!({}),
            |_input: serde_json::Value| async move {
                Ok(McpToolResult {
                    content: vec![ToolContent::Text {
                        text: "Error occurred".to_string(),
                    }],
                    is_error: true,
                })
            },
        );

        let result = tool.execute(serde_json::json!({})).await.unwrap();

        assert!(result.is_error);
        assert_eq!(result.content.len(), 1);
    }
}
