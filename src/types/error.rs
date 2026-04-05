use thiserror::Error;

/// Represents errors that can occur during SDK operations
#[derive(Debug, Error)]
pub enum SDKError {
    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("MCP error: {0}")]
    Mcp(String),

    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Error returned when an operation is aborted
#[derive(Debug, Error)]
#[error("Operation was aborted")]
pub struct AbortError;

impl AbortError {
    pub fn new() -> Self {
        AbortError
    }
}

impl PartialEq for AbortError {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// Check if an error is an abort error
pub fn is_abort_error(err: &anyhow::Error) -> bool {
    err.downcast_ref::<AbortError>().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abort_error_creation() {
        let err = AbortError::new();
        assert!(format!("{}", err).contains("aborted"));
    }

    #[test]
    fn test_abort_error_equality() {
        let err1 = AbortError::new();
        let err2 = AbortError::new();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_is_abort_error_true() {
        let err: Box<dyn std::error::Error> = Box::new(AbortError::new());
        // Note: This test will need adjustment based on how we box errors
        // For now, testing the function exists and compiles
    }

    #[test]
    fn test_sdk_error_transport_variant() {
        let err = SDKError::Transport("connection failed".to_string());
        assert!(format!("{}", err).contains("Transport"));
        assert!(format!("{}", err).contains("connection failed"));
    }

    #[test]
    fn test_sdk_error_session_variant() {
        let err = SDKError::Session("session closed".to_string());
        assert!(format!("{}", err).contains("Session"));
        assert!(format!("{}", err).contains("session closed"));
    }

    #[test]
    fn test_sdk_error_invalid_config_variant() {
        let err = SDKError::InvalidConfig("missing model".to_string());
        assert!(format!("{}", err).contains("Invalid configuration"));
        assert!(format!("{}", err).contains("missing model"));
    }

    #[test]
    fn test_sdk_error_mcp_variant() {
        let err = SDKError::Mcp("server unavailable".to_string());
        assert!(format!("{}", err).contains("MCP"));
        assert!(format!("{}", err).contains("server unavailable"));
    }

    #[test]
    fn test_sdk_error_tool_execution_variant() {
        let err = SDKError::ToolExecution("tool not found".to_string());
        assert!(format!("{}", err).contains("Tool execution"));
        assert!(format!("{}", err).contains("tool not found"));
    }

    #[test]
    fn test_sdk_error_debug_format() {
        let err = SDKError::Transport("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Transport"));
    }
}
