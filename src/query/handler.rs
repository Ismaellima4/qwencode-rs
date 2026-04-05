use anyhow::Result;
use tracing::info;

use crate::query::builder::QueryBuilder;
use crate::query::session::QueryHandle;
use crate::transport::stream::{create_message_stream, MessageHandler, MessageStream};
use crate::types::config::QueryOptions;
use crate::types::message::SDKMessage;

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

    // Create message stream
    let (handler, stream) = create_message_stream();

    // For now, return the result immediately
    // In full implementation, this would:
    // 1. Spawn the QwenCode CLI process
    // 2. Send the prompt via stdin
    // 3. Read responses from stdout
    // 4. Feed messages into the stream

    tracing::debug!("Query initialized with session: {}", handle.session_id());

    Ok(QueryResult {
        handle,
        stream,
        _handler: handler,
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

        let session_id = result.handle().session_id().to_string();
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
