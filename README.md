# qwencode-rs

Rust SDK for programmatic access to QwenCode CLI.

[![Crates.io](https://img.shields.io/crates/v/qwencode-rs.svg)](https://crates.io/crates/qwencode-rs)
[![Documentation](https://docs.rs/qwencode-rs/badge.svg)](https://docs.rs/qwencode-rs)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-151%20passing-brightgreen)]()
[![Clippy](https://img.shields.io/badge/clippy-clean-brightgreen)]()

## Overview

This SDK provides a Rust interface for interacting with the QwenCode CLI, enabling programmatic query execution, session management, and MCP (Model Context Protocol) server integration.

## Features

- **Async/Await API**: Built on Tokio for non-blocking operations
- **Type-safe**: Strong typing for all message types and configurations
- **Session Management**: Full control over query sessions
- **MCP Support**: Create and manage MCP servers with custom tools
- **Streaming**: Real-time message streaming with async channels
- **Builder Pattern**: Fluent API for easy configuration

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
qwencode-rs = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use qwencode_rs::{query, QueryOptions, SDKMessage};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let result = query(
        "What files are in the current directory?",
        QueryOptions::default()
    ).await?;

    while let Some(msg) = result.next_message().await {
        match msg? {
            SDKMessage::Assistant(a) => println!("Assistant: {}", a.message.content),
            SDKMessage::Result(r) => println!("Result: {:?}", r.result),
            _ => {}
        }
    }

    Ok(())
}
```

## Usage Examples

### Query with Custom Options

```rust
use qwencode_rs::{query, QueryOptions, PermissionMode};

let options = QueryOptions {
    model: Some("qwen-max".to_string()),
    permission_mode: PermissionMode::Yolo,
    debug: true,
    ..Default::default()
};

let result = query("Create a hello.txt file", options).await?;
```

### Using the Query Builder

```rust
use qwencode_rs::query_builder;

let result = query_builder()
    .prompt("Analyze this codebase")
    .model("qwen-plus")
    .cwd("/path/to/project")
    .permission_mode(PermissionMode::AutoEdit)
    .max_turns(10)
    .debug(true)
    .execute()
    .await?;
```

### Creating MCP Tools

```rust
use qwencode_rs::{tool, create_sdk_mcp_server, McpToolResult, ToolContent};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct AddArgs {
    a: i32,
    b: i32,
}

let add_tool = tool!(
    "add",
    "Add two numbers",
    AddArgs,
    |args: AddArgs| async move {
        Ok(McpToolResult {
            content: vec![ToolContent::Text {
                text: format!("{}", args.a + args.b),
            }],
            is_error: false,
        })
    }
);

let server = create_sdk_mcp_server("calculator", vec![add_tool]);
```

### Session Management

```rust
use qwencode_rs::{query, QueryOptions};

let options = QueryOptions {
    session_id: Some("my-session".to_string()),
    ..Default::default()
};

let result = query("Hello", options).await?;

// Get session ID
println!("Session: {}", result.handle().session_id());

// Interrupt the query
result.handle().interrupt().await?;

// Close the session
result.close().await?;
```

### Custom Permission Handler

```rust
use qwencode_rs::{query, QueryOptions, CanUseToolCallback, ToolPermissionResult};

async fn my_permission_handler(
    tool_name: String,
    input: serde_json::Value,
) -> anyhow::Result<ToolPermissionResult> {
    if tool_name.starts_with("read_") {
        return Ok(ToolPermissionResult::Allow {
            updated_input: input,
        });
    }
    
    // Custom logic
    Ok(ToolPermissionResult::Deny {
        message: "Not allowed".to_string(),
    })
}

let options = QueryOptions {
    // Note: can_use_tool field would be set here
    ..Default::default()
};
```

## API Reference

### Core Functions

- `query(prompt, options)` - Execute a query against QwenCode CLI
- `query_builder()` - Create a query using the fluent builder API
- `tool!(name, description, Args, handler)` - Macro to create MCP tools
- `create_sdk_mcp_server(name, tools)` - Create an MCP server

### Message Types

- `SDKMessage` - Enum of all message types
- `SDKUserMessage` - User messages
- `SDKAssistantMessage` - Assistant responses
- `SDKSystemMessage` - System messages
- `SDKResultMessage` - Query results
- `SDKPartialAssistantMessage` - Streaming partial messages

### Configuration

- `QueryOptions` - Main configuration struct
- `PermissionMode` - Permission modes (Default, Plan, AutoEdit, Yolo)
- `TimeoutConfig` - Timeout settings
- `AuthType` - Authentication types (OpenAI, Qwen OAuth)

### Error Handling

- `SDKError` - Main error enum
- `AbortError` - Cancellation error
- `is_abort_error(err)` - Check if error is abort-related

## Architecture

```
src/
├── lib.rs              # Public API exports
├── types/              # Type definitions
│   ├── message.rs      # SDK message types
│   ├── config.rs       # Configuration types
│   ├── error.rs        # Error types
│   ├── permission.rs   # Permission handling
│   └── mcp.rs          # MCP types
├── transport/          # Communication layer
│   ├── protocol.rs     # JSON-RPC protocol
│   ├── stream.rs       # Message streaming
│   └── stdin.rs        # Process management
├── query/              # Query logic
│   ├── session.rs      # Session management
│   ├── builder.rs      # Query builder
│   └── handler.rs      # Query execution
├── mcp/                # MCP support
│   ├── tool.rs         # Tool definitions
│   ├── server.rs       # MCP server
│   └── client.rs       # MCP client
└── utils/              # Utilities
    ├── validation.rs   # Validation helpers
    └── helpers.rs      # Utility functions
```

## Permission Modes

| Mode | Description |
|------|-------------|
| `Default` | Read tools auto-execute, write tools require approval |
| `Plan` | Only generate a plan, no tool execution |
| `AutoEdit` | Auto-approve edit and write_file tools |
| `Yolo` | Auto-approve all tools |

## Timeout Configuration

```rust
use qwencode_rs::TimeoutConfigBuilder;

let timeouts = TimeoutConfigBuilder::default()
    .can_use_tool(60000)     // 60 seconds
    .mcp_request(60000)      // 60 seconds
    .control_request(60000)  // 60 seconds
    .stream_close(15000)     // 15 seconds
    .build()
    .unwrap();
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific module tests
cargo test types::message

# Run with coverage (requires tarpaulin)
cargo tarpaulin
```

### Code Quality

```bash
# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Build documentation
cargo doc --no-deps --all-features --open
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Check compilation
cargo check --all-targets --all-features
```

### Best Practices

1. **TDD First**: Write tests before implementation
2. **Clippy Clean**: Zero warnings allowed (`-D warnings`)
3. **Format on Save**: Run `cargo fmt` before commits
4. **Semantic Versioning**: Follow semver for releases
5. **Conventional Commits**: Use commit message format

## CI/CD Pipeline

The project uses GitHub Actions for continuous integration:

### On Every Push/PR
- ✅ Format check
- ✅ Clippy linting
- ✅ Build & Test (Linux, macOS, Windows)
- ✅ MSRV check (Rust 1.75)
- ✅ Documentation build
- ✅ Security audit

### Release Process
1. Update version in `Cargo.toml`
2. Create and push tag: `git tag v0.1.1 && git push origin v0.1.1`
3. CI validates and publishes to crates.io
4. GitHub Release is created automatically

For details, see [`.github/workflows/`](.github/workflows/)

## Roadmap

- [x] Full CLI process integration
- [ ] Integration tests with mock CLI
- [ ] WebSocket transport support
- [ ] Advanced MCP server features
- [ ] Performance benchmarks
- [ ] Examples directory
- [ ] Publish to crates.io

## License

Apache-2.0

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Support

- Issues: https://github.com/Ismaellima4/qwencode-rs/issues
- Documentation: https://docs.rs/qwencode-rs
