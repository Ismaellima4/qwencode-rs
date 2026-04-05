use anyhow::Result;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::{timeout, Duration};
use tracing::{info, warn};

use crate::query::builder::QueryBuilder;
use crate::query::session::QueryHandle;
use crate::transport::stream::{create_message_stream, MessageHandler, MessageStream};
use crate::types::config::QueryOptions;
use crate::types::message::{
    MessageContent, MessageRole, SDKAssistantMessage, SDKMessage, SDKResultMessage,
};

/// Main query function - executes a query against QwenCode CLI
///
/// # Example
/// ```ignore
/// use qwencode_rs::query;
///
/// let result = query("What files are in the current directory?", QueryOptions::default()).await?;
/// while let Some(msg) = result.next_message().await {
///     match msg {
///         Ok(SDKMessage::Assistant(a)) => println!("Assistant: {}", a.message.content),
///         Ok(SDKMessage::Result(r)) => println!("Result: {:?}", r.result),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
pub async fn query(prompt: &str, options: QueryOptions) -> Result<QueryResult> {
    info!("Executing query: {}", prompt);

    // Create a session handle
    let handle = QueryHandle::new(options.session_id.clone());
    let session_id = handle.session_id().to_string();

    // Create message stream
    let (handler, stream) = create_message_stream();

    // Spawn task to handle CLI communication
    let handler_clone = handler.clone();
    let prompt_owned = prompt.to_string();
    tokio::spawn(async move {
        if let Err(e) = run_cli_session(&session_id, &prompt_owned, options, &handler_clone).await {
            warn!("CLI session error: {}", e);
            let _ = handler_clone
                .send_error(anyhow::anyhow!("CLI session error: {}", e))
                .await;
        }
        handler_clone.close();
    });

    tracing::debug!("Query initialized with session: {}", handle.session_id());

    Ok(QueryResult {
        handle,
        stream,
        _handler: handler,
    })
}

/// Run a CLI session: spawn process, send prompt, read responses
async fn run_cli_session(
    session_id: &str,
    prompt: &str,
    options: QueryOptions,
    handler: &MessageHandler,
) -> Result<()> {
    // Try to find qwen executable
    let executable_path = find_qwen_executable();

    match executable_path {
        Some(qwen_path) => {
            info!("Found Qwen CLI at: {}", qwen_path);
            run_real_cli_session(session_id, prompt, &options, qwen_path, handler).await
        }
        None => {
            // If qwen is not available, simulate a response
            info!("Qwen CLI not found in PATH, using simulated response");
            simulate_response(session_id, prompt, handler).await
        }
    }
}

/// Run a real CLI session with the qwen executable
async fn run_real_cli_session(
    session_id: &str,
    prompt: &str,
    options: &QueryOptions,
    executable_path: &str,
    handler: &MessageHandler,
) -> Result<()> {
    // Spawn the QwenCode CLI process in one-shot mode (non-interactive)
    let mut cmd = tokio::process::Command::new(executable_path);
    cmd.kill_on_drop(true)
        .stdin(std::process::Stdio::null()) // No stdin needed for one-shot mode
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    // Add the prompt as a positional argument (one-shot mode)
    cmd.arg(prompt);

    if let Some(cwd) = &options.cwd {
        cmd.current_dir(cwd);
    }

    if let Some(model) = &options.model {
        cmd.arg("--model").arg(model);
    }

    if options.debug {
        cmd.arg("--debug");
    }

    // Set channel to SDK for better identification
    cmd.arg("--channel").arg("SDK");

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            warn!(
                "Failed to spawn Qwen CLI: {}, falling back to simulation",
                e
            );
            return simulate_response(session_id, prompt, handler).await;
        }
    };

    info!("Qwen CLI spawned in one-shot mode, PID: {:?}", child.id());

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    // Read stdout line by line with timeout
    let mut stdout_reader = BufReader::new(stdout);
    let mut line = String::new();
    let idle_timeout = Duration::from_secs(60); // 60s timeout for one-shot mode

    loop {
        line.clear();
        match timeout(idle_timeout, stdout_reader.read_line(&mut line)).await {
            Ok(Ok(0)) => break, // EOF
            Ok(Ok(_)) => {
                let line = line.trim().to_string();
                if !line.is_empty() {
                    info!("CLI output: {}", line);
                    let assistant_msg = SDKMessage::Assistant(SDKAssistantMessage {
                        session_id: session_id.to_string(),
                        message: MessageContent {
                            role: MessageRole::Assistant,
                            content: line.clone(),
                        },
                    });
                    if let Err(e) = handler.send_message(assistant_msg).await {
                        warn!("Failed to send message: {}", e);
                        break;
                    }
                }
            }
            Ok(Err(e)) => {
                warn!("Error reading stdout: {}", e);
                break;
            }
            Err(_) => {
                info!(
                    "Idle timeout reached ({}s), ending session",
                    idle_timeout.as_secs()
                );
                break;
            }
        }
    }

    // Kill the child process if still running
    let _ = child.start_kill();

    // Send result message
    let result_msg = SDKMessage::Result(SDKResultMessage {
        session_id: session_id.to_string(),
        result: serde_json::json!({
            "exit_code": 0,
            "success": true,
            "note": "Process completed (killed after idle timeout)"
        }),
        exit_code: 0,
    });
    handler.send_message(result_msg).await?;

    // Log stderr if any
    let mut stderr_reader = BufReader::new(stderr);
    let mut stderr_line = String::new();
    loop {
        stderr_line.clear();
        match stderr_reader.read_line(&mut stderr_line).await {
            Ok(0) => break,
            Ok(_) => {
                if !stderr_line.trim().is_empty() {
                    warn!("[stderr] {}", stderr_line.trim());
                }
            }
            Err(_) => break,
        }
    }

    Ok(())
}

/// Simulate a response when qwen CLI is not available
async fn simulate_response(session_id: &str, prompt: &str, handler: &MessageHandler) -> Result<()> {
    info!("Qwen CLI not found, simulating response");

    // Simulate assistant thinking
    let thinking_msg = SDKMessage::Assistant(SDKAssistantMessage {
        session_id: session_id.to_string(),
        message: MessageContent {
            role: MessageRole::Assistant,
            content: format!(
                "[Simulated] Processing query: \"{}\"\n\nSince the QwenCode CLI is not installed, this is a simulated response. To get real responses, install the QwenCode CLI and ensure it's in your PATH.",
                prompt
            ),
        },
    });
    handler.send_message(thinking_msg).await?;

    // Send result
    let result_msg = SDKMessage::Result(SDKResultMessage {
        session_id: session_id.to_string(),
        result: serde_json::json!({
            "status": "simulated",
            "note": "Install QwenCode CLI for real responses"
        }),
        exit_code: 0,
    });
    handler.send_message(result_msg).await?;

    Ok(())
}

/// Find the qwen executable in PATH
fn find_qwen_executable() -> Option<&'static str> {
    // Try common executable names
    ["qwen", "qwen-code"]
        .iter()
        .find(|&name| which(name).is_some())
        .copied()
}

/// Check if an executable exists in PATH
fn which(executable: &str) -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join(executable);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    })
}

/// Query result containing session handle and message stream
pub struct QueryResult {
    handle: QueryHandle,
    stream: MessageStream,
    _handler: MessageHandler,
}

impl QueryResult {
    /// Get the session handle
    pub fn handle(&self) -> &QueryHandle {
        &self.handle
    }

    /// Get the message stream
    pub fn stream(&self) -> &MessageStream {
        &self.stream
    }

    /// Get the next message from the stream
    pub async fn next_message(&self) -> Option<Result<SDKMessage>> {
        self.stream.next_message().await
    }

    /// Close the query
    pub async fn close(mut self) -> Result<()> {
        self.handle.close().await
    }
}

/// Query builder function for fluent API
///
/// # Example
/// ```ignore
/// use qwencode_rs::query_builder;
///
/// let result = query_builder()
///     .prompt("Hello")
///     .model("qwen-max")
///     .debug(true)
///     .execute()
///     .await?;
/// ```
pub fn query_builder() -> QueryBuilder {
    QueryBuilder::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_initial_state() {
        let result = query("Test prompt", QueryOptions::default()).await.unwrap();

        assert!(!result.handle().is_closed());
        assert!(!result.stream().is_closed());
    }

    #[tokio::test]
    async fn test_query_session_id() {
        let options = QueryOptions {
            session_id: Some("custom-session".to_string()),
            ..Default::default()
        };

        let result = query("Test", options).await.unwrap();
        assert_eq!(result.handle().session_id(), "custom-session");
    }

    #[tokio::test]
    async fn test_query_close() {
        let result = query("Test", QueryOptions::default()).await.unwrap();

        let _session_id = result.handle().session_id().to_string();
        result.close().await.unwrap();

        // Session should be closed
        // Note: We can't check handle directly after move, but close should succeed
    }

    #[test]
    fn test_query_builder_function() {
        let builder = query_builder();
        assert!(builder.prompt.is_none());
    }

    #[tokio::test]
    async fn test_query_with_custom_options() {
        let options = QueryOptions {
            model: Some("qwen-plus".to_string()),
            debug: true,
            max_session_turns: 10,
            ..Default::default()
        };

        let result = query("Test with options", options).await.unwrap();
        assert!(!result.handle().is_closed());
    }
}
