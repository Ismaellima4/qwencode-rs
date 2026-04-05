# Project Architecture: qwencode-rs

## Module Structure

```
src/
├── lib.rs                  # Public API, re-exports
├── types/                  # Type definitions
│   ├── mod.rs             # Module exports
│   ├── message.rs         # SDK message types
│   ├── config.rs          # QueryOptions and configuration
│   ├── error.rs           # Error types (AbortError, SDKError)
│   ├── permission.rs      # Permission modes and tool handling
│   └── mcp.rs             # MCP-related types
├── transport/              # Communication layer
│   ├── mod.rs             # Module exports
│   ├── stdin.rs           # stdin/stdout communication
│   ├── stream.rs          # Message stream handling
│   └── protocol.rs        # Communication protocol
├── query/                  # Query logic
│   ├── mod.rs             # Module exports
│   ├── session.rs         # Session management
│   ├── builder.rs         # Query builder pattern
│   └── handler.rs         # Message handlers
├── mcp/                    # MCP server support
│   ├── mod.rs             # Module exports
│   ├── server.rs          # Embedded MCP server
│   ├── tool.rs            # Tool definitions
│   └── client.rs          # MCP client
└── utils/                  # Utilities
    ├── mod.rs             # Module exports
    ├── validation.rs      # Validation helpers
    └── helpers.rs         # Utility functions
```

## Key Design Decisions

### 1. Async Runtime
- **Tokio** as the async runtime
- All public APIs use async/await
- Stream-based message handling via `tokio-stream`

### 2. Error Handling
- `thiserror` for custom error types
- `anyhow` for application-level errors
- Clear distinction between recoverable and fatal errors

### 3. Builder Pattern
- All complex config structs use derive_builder
- Fluent API for configuration
- Sensible defaults with override capability

### 4. Type Safety
- Strong typing for all message types
- Enum-based message discrimination
- Pattern matching for type guards (idiomatic Rust)

### 5. MCP Integration
- Compatible with MCP SDK protocol
- Support for both external and embedded servers
- Tool definition with schema validation via schemars

## API Stability

### Public API (lib.rs exports)
```rust
// Core functions
pub fn query();
pub fn query_stream();
pub fn tool!();
pub fn create_sdk_mcp_server();

// Types
pub use types::{SDKMessage, QueryOptions, PermissionMode, ...};

// Error types
pub use types::{SDKError, AbortError};

// MCP types
pub use mcp::{McpServerConfig, McpToolResult, ...};
```

### Internal API (not exported)
- transport::*
- query::* (except QueryHandle)
- mcp internals
- utils::*
