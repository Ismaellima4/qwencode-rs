# Project Specification: qwencode-rs

## 📋 Overview

Rust SDK for programmatic access to QwenCode CLI, based on the official TypeScript SDK (`@qwen-code/sdk`).

### Objectives
- Provide an idiomatic Rust async API (async/await with Tokio)
- Maintain feature parity with the TypeScript SDK
- Leverage Rust's strong type system for compile-time safety
- Support stdin/stdout communication with QwenCode CLI
- Integrate with MCP (Model Context Protocol) servers

## 🏗️ Architecture

### Module Structure

```
src/
├── lib.rs              # Main public API, re-exports
├── types/              # Type and structure definitions
│   ├── mod.rs
│   ├── message.rs      # Message types (User, Assistant, System, Result)
│   ├── config.rs       # QueryOptions and configuration
│   ├── error.rs        # Error types (AbortError, SDKError)
│   ├── permission.rs   # Permission modes and CanUseTool
│   └── mcp.rs          # MCP-related types
├── transport/          # Communication layer
│   ├── mod.rs
│   ├── stdin.rs        # Process spawning utilities
│   ├── stream.rs       # Message stream handling
│   └── protocol.rs     # Communication protocol
├── query/              # Main query logic
│   ├── mod.rs
│   ├── session.rs      # Session management
│   ├── builder.rs      # Query builder pattern
│   └── handler.rs      # Query execution with CLI integration
├── mcp/                # MCP server support
│   ├── mod.rs
│   ├── server.rs       # Embedded MCP server
│   ├── tool.rs         # Tool definitions
│   └── client.rs       # MCP client
└── utils/              # Utilities
    ├── mod.rs
    ├── validation.rs   # Validation helpers
    └── helpers.rs      # Utility functions
```

## 📦 Main Dependencies

### Production
```toml
[dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }
tokio-util = { version = "0.7", features = ["codec"] }
tokio-stream = "0.1"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# MCP SDK
reqwest = { version = "0.11", features = ["json", "stream"] }

# Validation/Schema
schemars = "0.8"  # Equivalent to Zod in Rust

# Errors
thiserror = "1"
anyhow = "1"

# Logging/Debug
tracing = "0.1"

# UUID for session IDs
uuid = { version = "1", features = ["v4"] }

# Channels for communication
async-channel = "2"

# Timeout
tokio-util = { version = "0.7", features = ["time"] }
```

### Development
```toml
[dev-dependencies]
tokio-test = "0.4"
tempfile = "3"
assert_fs = "1"
tracing-subscriber = "0.3"
```

## 🔧 API Design

### Main Function: `query()`

```rust
use qwencode_rs::{query, QueryOptions, SDKMessage};

let options = QueryOptions::builder()
    .cwd("/path/to/project")
    .model("qwen-max")
    .permission_mode(PermissionMode::Default)
    .build()?;

let mut result = query("What files are in the current directory?", options).await?;

while let Some(message) = result.next().await {
    match message {
        SDKMessage::Assistant(msg) => println!("Assistant: {}", msg.content),
        SDKMessage::Result(msg) => println!("Result: {:?}", msg.result),
        _ => {}
    }
}
```

### Message Types

```rust
pub enum SDKMessage {
    User(SDKUserMessage),
    Assistant(SDKAssistantMessage),
    System(SDKSystemMessage),
    Result(SDKResultMessage),
    PartialAssistant(SDKPartialAssistantMessage),
}

// Type guards via idiomatic Rust pattern matching
```

### QueryOptions

```rust
#[derive(Debug, Clone, Builder)]
pub struct QueryOptions {
    pub cwd: Option<PathBuf>,
    pub model: Option<String>,
    pub path_to_qwen_executable: Option<String>,
    pub permission_mode: PermissionMode,
    pub can_use_tool: Option<CanUseToolCallback>,
    pub env: Option<HashMap<String, String>>,
    pub system_prompt: Option<SystemPromptConfig>,
    pub mcp_servers: Option<HashMap<String, McpServerConfig>>,
    pub abort_signal: Option<tokio_util::sync::CancellationToken>,
    pub debug: bool,
    pub max_session_turns: Option<i32>,
    pub core_tools: Option<Vec<String>>,
    pub exclude_tools: Option<Vec<String>>,
    pub allowed_tools: Option<Vec<String>>,
    pub auth_type: AuthType,
    pub agents: Option<Vec<SubagentConfig>>,
    pub include_partial_messages: bool,
    pub resume: Option<String>,
    pub session_id: Option<String>,
    pub timeouts: Option<TimeoutConfig>,
}
```

### PermissionMode

```rust
pub enum PermissionMode {
    Default,    // Auto-read, write requires approval
    Plan,       // Plan only, blocks writes
    AutoEdit,   // Auto-approve edits
    Yolo,       // Auto-approve everything
}
```

### Query Handle Methods

```rust
pub struct QueryHandle {
    // internal fields
}

impl QueryHandle {
    pub fn session_id(&self) -> &str;
    pub fn is_closed(&self) -> bool;
    pub async fn interrupt(&self) -> Result<()>;
    pub async fn set_permission_mode(&self, mode: PermissionMode) -> Result<()>;
    pub async fn set_model(&self, model: String) -> Result<()>;
    pub async fn close(self) -> Result<()>;
}
```

### MCP Tool Definition

```rust
use qwencode_rs::{tool, create_sdk_mcp_server};

let calc_tool = tool!(
    "calculate_sum",
    "Add two numbers",
    |args: CalcArgs| async move {
        MCPToolResult {
            content: vec![ToolContent::Text {
                text: format!("{}", args.a + args.b),
            }],
        }
    }
);

let server = create_sdk_mcp_server("calculator", vec![calc_tool]);
```

### Abort with CancellationToken

```rust
use tokio_util::sync::CancellationToken;

let cancel_token = CancellationToken::new();
let cancel_token_clone = cancel_token.clone();

tokio::spawn(async move {
    tokio::time::sleep(Duration::from_secs(5)).await;
    cancel_token_clone.cancel();
});

let result = query("Long task...", options).await;
// CancellationToken::is_cancelled() indicates abort
```

## 🎯 TypeScript → Rust Translation

| TypeScript | Rust |
|------------|------|
| `AbortController` | `tokio_util::sync::CancellationToken` |
| `AsyncIterable<T>` | `impl Stream<Item = T>` or `AsyncIterator` |
| `Promise<T>` | `Future<Output = T>` |
| `zod` schemas | `schemars` + manual validation |
| Type guards | Pattern matching (`match`) |
| `Record<string, T>` | `HashMap<String, T>` |
| Callbacks | Closures + `Box<dyn Fn...>` |
| `export { ... }` | `pub use ...` |

## ⏱️ Default Timeouts

| Timeout | Default | Description |
|---------|---------|-------------|
| `can_use_tool` | 60s | Permission callback timeout |
| `mcp_request` | 60s | MCP tool call timeout |
| `control_request` | 60s | `initialize()`, `set_model()`, etc. |
| `stream_close` | 15s | stdin close timeout in multi-turn mode |

## 🔐 Permission Modes

### Priority Chain
```
exclude_tools/deny
  > ask
  > plan
  > yolo
  > allowed_tools/allow
  > can_use_tool callback
  > default behavior
```

## 🚀 Usage Examples

### Example 1: Simple single-turn
```rust
let result = query("Create a hello.txt file", QueryOptions::default()).await?;
while let Some(msg) = result.next().await {
    if let SDKMessage::Assistant(a) = msg {
        println!("{}", a.content);
    }
}
```

### Example 2: Multi-turn with Stream
```rust
async fn message_stream() -> impl Stream<Item = SDKUserMessage> {
    let (tx, rx) = async_channel::unbounded();

    tx.send(SDKUserMessage { content: "Create hello.txt".into(), ..Default::default() }).await.unwrap();
    // ... more messages

    rx
}

let mut result = query_stream(message_stream().await, options).await?;
```

### Example 3: Custom Tool Handler
```rust
async fn can_use_tool(
    tool_name: &str,
    input: &serde_json::Value,
) -> ToolPermissionResult {
    if tool_name.starts_with("read_") {
        return ToolPermissionResult::Allow { input: input.clone() };
    }

    // Custom approval logic
    ToolPermissionResult::Deny { message: "Denied".into() }
}

let options = QueryOptions::builder()
    .can_use_tool(can_use_tool)
    .build()?;
```

### Example 4: Embedded MCP Server
```rust
let server = create_sdk_mcp_server(McpServerOptions {
    name: "calculator".into(),
    tools: vec![calc_tool],
});

let options = QueryOptions::builder()
    .permission_mode(PermissionMode::Yolo)
    .mcp_server("calculator", server)
    .build()?;
```

## 🧪 Testing Strategy

### Unit Tests
- Type and configuration validation
- Communication protocol
- MCP tool definitions
- Permission handling logic

### Integration Tests
- stdin/stdout communication with mock CLI
- MCP server integration
- Session management
- Abort/cancellation

### Example Tests
- All documentation examples should compile and run
- Use `#[doc_test]` or `doctest`

## 📚 Documentation

### README.md should include:
1. Installation (Cargo.toml dependency)
2. Quick Start
3. API Reference
4. Practical examples
5. Error handling
6. FAQ

### Rustdoc
- Document all public types and functions
- Include examples in each type/function
- Links to related documentation

## 🔄 Compatibility

### Minimum Versions
- Rust: 1.86+ (required by icu_* dependencies via reqwest -> url -> idna)
- Tokio: 1.x
- MSRV (Minimum Supported Rust Version): 1.86

### Platforms
- Linux
- macOS
- Windows (msvc/gnu)

## 📊 Success Metrics

- [x] Feature parity with TypeScript SDK v0.1.6
- [x] All TypeScript examples ported to Rust
- [ ] Test coverage > 80%
- [x] Complete documentation with examples
- [x] Clean clippy build (zero warnings)
- [ ] Performance benchmarks vs TypeScript

## 🗓️ Next Steps

1. ✅ Project specification (this document)
2. ✅ Configure Cargo.toml with dependencies
3. ✅ Create module structure (types, transport, query, mcp, utils)
4. ✅ Implement main types
5. ✅ Implement transport layer
6. ✅ Implement query engine
7. ✅ Implement MCP support
8. ✅ Add tests (151 tests passing)
9. ✅ Create basic documentation
10. ✅ Fix all clippy warnings (--all-targets --all-features -- -D warnings)
11. ✅ Implement real CLI communication (one-shot mode)
12. ✅ Add usage examples (basic_query, mcp_server)
13. 🔄 Integration tests with mock CLI
14. 📋 Publish to crates.io

## ✅ Current Status

- **Tests**: 151 unit tests passing
- **Clippy**: Clean (--all-targets --all-features -- -D warnings)
- **Modules**: All implemented and compiling
- **CLI Communication**: ✅ Implemented (spawn, one-shot mode, stream output, graceful fallback)
- **Next**: Integration tests with mock CLI
