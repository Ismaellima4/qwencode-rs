use anyhow::{Context, Result};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::transport::communication::{protocol_to_sdk_message, spawn_cli_process, CLIRequest};
use crate::transport::stream::{create_message_stream, MessageStream};
use crate::types::config::QueryOptions;
use crate::types::message::SDKMessage;

/// Execute a query against the QwenCode CLI with full process communication
///
/// This spawns the CLI process, sends the query via stdin, and reads
/// responses from stdout into a message stream.
pub async fn execute_query(prompt: &str, options: QueryOptions) -> Result<QueryResultWithCLI> {
    info!("Executing query with CLI communication: {}", prompt);

    // Spawn CLI process
    let process = spawn_cli_process(options.path_to_qwen_executable.as_deref())
        .await
        .context("Failed to spawn CLI process")?;

    // Create message stream
    let (handler, stream) = create_message_stream();
    let cancel_token = CancellationToken::new();

    // Initialize connection (best effort)
    let mut process = process;
    let cancel_token_init = cancel_token.clone();
    match process.initialize(&cancel_token_init).await {
        Ok(response) => {
            debug!(
                "CLI initialized: protocol_version={}, streaming={}",
                response.protocol_version, response.capabilities.streaming
            );
        }
        Err(e) => {
            warn!(
                "CLI initialization failed (expected if CLI doesn't support init): {}",
                e
            );
        }
    }

    // Create request
    let request = CLIRequest {
        request_type: "query".to_string(),
        prompt: prompt.to_string(),
        session_id: options.session_id.clone(),
        options: options.clone(),
    };

    // Send query
    process
        .send_query(&request)
        .await
        .context("Failed to send query")?;

    // Spawn message reader task
    let handler_for_task = handler;
    let cancel_token_task = cancel_token.clone();

    tokio::spawn(async move {
        let mut process = process;
        let handler = handler_for_task;

        loop {
            tokio::select! {
                _ = cancel_token_task.cancelled() => {
                    debug!("Query cancelled");
                    break;
                }
                result = process.read_message() => {
                    match result {
                        Ok(Some(msg)) => {
                            match protocol_to_sdk_message(&msg) {
                                Ok(Some(sdk_msg)) => {
                                    if let Err(e) = handler.send_message(sdk_msg).await {
                                        error!("Failed to send message to stream: {}", e);
                                    }
                                }
                                Ok(None) => {
                                    debug!("Message filtered out");
                                }
                                Err(e) => {
                                    error!("Error converting message: {}", e);
                                    if let Err(e) = handler.send_error(e).await {
                                        error!("Failed to send error to stream: {}", e);
                                    }
                                }
                            }
                        }
                        Ok(None) => {
                            debug!("CLI process exited");
                            break;
                        }
                        Err(e) => {
                            error!("Error reading from CLI: {}", e);
                            if let Err(e) = handler.send_error(e).await {
                                error!("Failed to send error to stream: {}", e);
                            }
                            break;
                        }
                    }
                }
            }
        }

        // Cleanup
        if let Err(e) = process.shutdown().await {
            warn!("Error during CLI shutdown: {}", e);
        }
    });

    // Create query handle
    let handle = crate::query::session::QueryHandle::new(options.session_id.clone());

    Ok(QueryResultWithCLI {
        handle,
        stream,
        cancel_token,
    })
}

/// Query result containing session handle and message stream (with CLI process)
pub struct QueryResultWithCLI {
    pub handle: crate::query::session::QueryHandle,
    pub stream: MessageStream,
    pub cancel_token: CancellationToken,
}

impl QueryResultWithCLI {
    /// Get the session handle
    pub fn handle(&self) -> &crate::query::session::QueryHandle {
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

    /// Cancel the query
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    /// Close the query
    pub async fn close(self) -> Result<()> {
        self.cancel_token.cancel();
        let mut this = self;
        this.handle.close().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_result_with_cli_creation() {
        // Test QueryResultWithCLI can be created (without actual CLI)
        let (handler, stream) = create_message_stream();
        let handle = crate::query::session::QueryHandle::new(None);
        let cancel_token = CancellationToken::new();

        let result = QueryResultWithCLI {
            handle,
            stream,
            cancel_token: cancel_token.clone(),
        };

        assert!(!result.handle().is_closed());
        // Keep handler alive to prevent stream from closing
        drop(handler);
    }

    #[tokio::test]
    async fn test_query_result_cancel() {
        let (_, stream) = create_message_stream();
        let handle = crate::query::session::QueryHandle::new(None);
        let cancel_token = CancellationToken::new();

        let result = QueryResultWithCLI {
            handle,
            stream,
            cancel_token: cancel_token.clone(),
        };

        result.cancel();
        assert!(cancel_token.is_cancelled());
    }

    #[tokio::test]
    async fn test_query_result_close() {
        let (_, stream) = create_message_stream();
        let handle = crate::query::session::QueryHandle::new(Some("test-session".to_string()));
        let cancel_token = CancellationToken::new();

        let result = QueryResultWithCLI {
            handle,
            stream,
            cancel_token: cancel_token.clone(),
        };

        result.close().await.unwrap();
        assert!(cancel_token.is_cancelled());
    }
}
