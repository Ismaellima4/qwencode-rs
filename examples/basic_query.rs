//! Basic query example using qwencode-rs
//!
//! This example demonstrates how to:
//! - Create a query using the query builder
//! - Configure query options (model, working directory, permission mode)
//! - Execute a query and handle responses
//!
//! Run with: `cargo run --example basic_query`

use qwencode_rs::types::permission::PermissionMode;
use qwencode_rs::{query, query_builder, SDKMessage};

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

    // Stream and display messages
    println!("📨 Receiving messages...");
    println!("{}", "─".repeat(60));

    let mut message_count = 0;

    loop {
        match result.next_message().await {
            Some(Ok(msg)) => {
                message_count += 1;
                match &msg {
                    SDKMessage::Assistant(a) => {
                        println!("🤖 Assistant:");
                        println!("   {}", a.message.content);
                    }
                    SDKMessage::Result(r) => {
                        println!();
                        println!("✅ Result:");
                        println!("   Exit code: {}", r.exit_code);
                        println!(
                            "   Success: {}",
                            r.result
                                .get("success")
                                .unwrap_or(&serde_json::Value::Bool(false))
                        );
                    }
                    SDKMessage::User(u) => {
                        println!("👤 User: {}", u.message.content);
                    }
                    SDKMessage::System(s) => {
                        println!("⚙️  System: {}", s.message.content);
                    }
                    SDKMessage::PartialAssistant(p) => {
                        println!("⏳ Partial: {}", p.message.content);
                    }
                }
                println!();
            }
            Some(Err(e)) => {
                eprintln!("❌ Error: {}", e);
                break;
            }
            None => {
                println!("🏁 Stream ended");
                break;
            }
        }
    }

    println!("{}", "─".repeat(60));
    println!("📊 Total messages: {}", message_count);
    println!("✨ Done!");

    Ok(())
}
