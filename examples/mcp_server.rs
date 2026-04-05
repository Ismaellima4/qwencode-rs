//! MCP Server example using qwencode-rs
//!
//! This example demonstrates how to:
//! - Create an MCP server
//! - Register custom tools
//! - Execute tools
//!
//! Run with: `cargo run --example mcp_server`

use qwencode_rs::mcp::tool::McpTool;
use qwencode_rs::types::mcp::{McpToolResult, ToolContent};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 qwencode-rs - MCP Server Example\n");

    // Create an MCP server
    let server = qwencode_rs::mcp::server::SdkMcpServer::new("my-custom-server");
    println!("✅ Server created: {}", server.name);
    println!();

    // Register a custom tool: File Reader
    println!("🔧 Registering tools...");

    let file_reader = McpTool::new(
        "file_reader",
        "Read the contents of a file",
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file"
                }
            },
            "required": ["path"]
        }),
        |input: serde_json::Value| async move {
            let path = input["path"].as_str().unwrap_or("unknown");
            Ok(McpToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Reading file: {}", path),
                }],
                is_error: false,
            })
        },
    );

    server.register_tool(file_reader).await;
    println!("  ✅ file_reader registered");

    // Register a custom tool: Code Analyzer
    let code_analyzer = McpTool::new(
        "code_analyzer",
        "Analyze code for issues",
        serde_json::json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "Code to analyze"
                },
                "language": {
                    "type": "string",
                    "description": "Programming language"
                }
            },
            "required": ["code"]
        }),
        |input: serde_json::Value| async move {
            let code = input["code"].as_str().unwrap_or("");
            let language = input["language"].as_str().unwrap_or("unknown");
            let issues_count = code.lines().count();

            Ok(McpToolResult {
                content: vec![ToolContent::Text {
                    text: format!(
                        "Analyzed {} code: {} lines, found {} potential issues",
                        language,
                        issues_count,
                        issues_count / 10
                    ),
                }],
                is_error: false,
            })
        },
    );

    server.register_tool(code_analyzer).await;
    println!("  ✅ code_analyzer registered");
    println!();

    // List registered tools
    let tool_names = server.get_tool_names().await;
    println!("📋 Registered tools: {:?}", tool_names);
    println!("🔢 Total tools: {}", server.tool_count().await);
    println!();

    // Execute tools
    println!("⚡ Executing tools...");

    // Execute file_reader
    let result = server
        .execute_tool(
            "file_reader",
            serde_json::json!({
                "path": "examples/mcp_server.rs"
            }),
        )
        .await?;

    println!("📄 file_reader result:");
    for content in &result.content {
        if let ToolContent::Text { text } = content {
            println!("   {}", text);
        }
    }
    println!();

    // Execute code_analyzer
    let result = server
        .execute_tool(
            "code_analyzer",
            serde_json::json!({
                "code": "fn main() {\n    println!(\"Hello\");\n}",
                "language": "Rust"
            }),
        )
        .await?;

    println!("🔍 code_analyzer result:");
    for content in &result.content {
        if let ToolContent::Text { text } = content {
            println!("   {}", text);
        }
    }
    println!();

    println!("✨ Done!");

    Ok(())
}
