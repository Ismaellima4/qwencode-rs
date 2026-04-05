//! Basic query example using qwencode-rs
//!
//! This example demonstrates how to:
//! - Create a query using the query builder
//! - Configure query options (model, working directory, permission mode)
//! - Execute a query and handle responses
//!
//! Run with: `cargo run --example basic_query`

use qwencode_rs::types::permission::PermissionMode;
use qwencode_rs::{query, query_builder};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing (optional, for debugging)
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 qwencode-rs - Basic Query Example\n");

    // Build query options using the fluent API
    let (prompt, options) = query_builder()
        .prompt("What files are in the current directory?")
        .cwd(".")
        .model("qwen-coder")
        .permission_mode(PermissionMode::Plan)
        .max_turns(5)
        .debug(false)
        .build()?;

    println!("📝 Prompt: {}", prompt.as_deref().unwrap_or("(none)"));
    println!("📁 Working directory: {:?}", options.cwd);
    println!("🤖 Model: {:?}", options.model);
    println!("🔐 Permission mode: {:?}", options.permission_mode);
    println!("🔄 Max turns: {:?}", options.max_session_turns);
    println!();

    // Execute the query
    println!("⏳ Executing query...");
    let result = query("What files are in the current directory?", options).await?;

    println!("✅ Query executed successfully!");
    println!("📋 Session ID: {}", result.handle().session_id());
    println!();

    // Note: The current implementation is a placeholder.
    // In the full implementation, this would:
    // 1. Spawn the QwenCode CLI process
    // 2. Send the prompt via stdin
    // 3. Read responses from stdout
    // 4. Stream messages in real-time
    println!("📨 Message streaming (placeholder)...");
    println!("   ⚠️  Full CLI integration not yet implemented");
    println!("   📌 Session is ready and waiting for messages");
    println!();

    println!("📊 Session status:");
    println!("   - Active: {}", !result.handle().is_closed());
    println!("   - Stream open: {}", !result.stream().is_closed());
    println!();
    println!("✨ Done!");

    Ok(())
}
