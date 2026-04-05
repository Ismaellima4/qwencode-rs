//! # qwencode-rs
//!
//! Rust SDK for programmatic access to QwenCode CLI.
//!
//! ## Quick Start
//!
//! ```ignore
//! use qwencode_rs::{query, QueryOptions, SDKMessage};
//!
//! let result = query("What files are in the current directory?", QueryOptions::default()).await?;
//! while let Some(msg) = result.next_message().await {
//!     match msg {
//!         Ok(SDKMessage::Assistant(a)) => println!("Assistant: {}", a.message.content),
//!         Ok(SDKMessage::Result(r)) => println!("Result: {:?}", r.result),
//!         _ => {}
//!     }
//! }
//! ```

pub mod mcp;
pub mod query;
pub mod transport;
pub mod types;
pub mod utils;

// Re-export main public API
pub use types::config::*;
pub use types::error::*;
pub use types::mcp::*;
pub use types::message::*;
pub use types::permission::*;

pub use query::builder::QueryBuilder;
pub use query::handler::{query, query_builder, QueryResult};
pub use query::session::QueryHandle;

pub use mcp::client::McpClient;
pub use mcp::server::{create_sdk_mcp_server, SdkMcpServer};
pub use mcp::tool::McpTool;
