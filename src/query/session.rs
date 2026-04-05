use anyhow::Result;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};
use uuid::Uuid;

use crate::types::config::QueryOptions;
use crate::types::permission::PermissionMode;

/// Handle for an active query session
pub struct QueryHandle {
    session_id: String,
    cancel_token: CancellationToken,
    is_closed: bool,
}

impl QueryHandle {
    /// Create a new query handle with a session ID
    pub fn new(session_id: Option<String>) -> Self {
        let session_id = session_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        debug!("Creating new QueryHandle with session_id: {}", session_id);

        QueryHandle {
            session_id,
            cancel_token: CancellationToken::new(),
            is_closed: false,
        }
    }

    /// Get the session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Check if the session is closed
    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    /// Get the cancellation token
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    /// Interrupt the current operation
    pub async fn interrupt(&self) -> Result<()> {
        info!("Interrupting session {}", self.session_id);
        self.cancel_token.cancel();
        Ok(())
    }

    /// Set the permission mode (would update the running CLI process)
    pub async fn set_permission_mode(&self, _mode: PermissionMode) -> Result<()> {
        // TODO: Implement mode change via CLI communication
        info!("Setting permission mode for session {}", self.session_id);
        Ok(())
    }

    /// Set the model (would update the running CLI process)
    pub async fn set_model(&self, _model: &str) -> Result<()> {
        // TODO: Implement model change via CLI communication
        info!("Setting model for session {}", self.session_id);
        Ok(())
    }

    /// Close the session
    pub async fn close(&mut self) -> Result<()> {
        if !self.is_closed {
            info!("Closing session {}", self.session_id);
            self.cancel_token.cancel();
            self.is_closed = true;
        }
        Ok(())
    }
}

impl Drop for QueryHandle {
    fn drop(&mut self) {
        if !self.is_closed {
            debug!("QueryHandle dropped without explicit close, cancelling");
            self.cancel_token.cancel();
        }
    }
}

/// Generate a session ID
pub fn generate_session_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_handle_creation() {
        let handle = QueryHandle::new(None);

        assert!(!handle.session_id().is_empty());
        assert!(!handle.is_closed());
    }

    #[test]
    fn test_query_handle_with_session_id() {
        let handle = QueryHandle::new(Some("custom-session-id".to_string()));

        assert_eq!(handle.session_id(), "custom-session-id");
    }

    #[test]
    fn test_query_handle_cancellation_token() {
        let handle = QueryHandle::new(None);
        let token = handle.cancellation_token();

        assert!(!token.is_cancelled());
    }

    #[tokio::test]
    async fn test_query_handle_interrupt() {
        let handle = QueryHandle::new(None);
        let token = handle.cancellation_token();

        handle.interrupt().await.unwrap();
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn test_query_handle_close() {
        let mut handle = QueryHandle::new(None);

        assert!(!handle.is_closed());
        handle.close().await.unwrap();
        assert!(handle.is_closed());
    }

    #[tokio::test]
    async fn test_query_handle_close_multiple_times() {
        let mut handle = QueryHandle::new(None);

        handle.close().await.unwrap();
        handle.close().await.unwrap(); // Should not panic

        assert!(handle.is_closed());
    }

    #[test]
    fn test_generate_session_id() {
        let id1 = generate_session_id();
        let id2 = generate_session_id();

        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_session_id_format() {
        let id = generate_session_id();

        // UUID v4 format: 8-4-4-4-12 hexadecimal characters
        let parts: Vec<&str> = id.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);
    }

    #[tokio::test]
    async fn test_query_handle_set_permission_mode() {
        let handle = QueryHandle::new(None);

        let result = handle.set_permission_mode(PermissionMode::Yolo).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_handle_set_model() {
        let handle = QueryHandle::new(None);

        let result = handle.set_model("qwen-max").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_query_handle_drop_cancels_token() {
        let token;
        {
            let handle = QueryHandle::new(None);
            token = handle.cancellation_token();
            assert!(!token.is_cancelled());
        } // handle dropped

        // After drop, token should be cancelled
        assert!(token.is_cancelled());
    }
}
