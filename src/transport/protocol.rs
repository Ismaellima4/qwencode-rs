use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout};
use tracing::{debug, info};

/// Protocol message for stdin/stdout communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub id: Option<u64>,
    pub jsonrpc: String,
    pub method: Option<String>,
    pub params: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<ProtocolError>,
}

/// Protocol error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolError {
    pub code: i64,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Stdin transport handler
pub struct StdinTransport {
    stdin: ChildStdin,
    stdout: ChildStdout,
}

impl StdinTransport {
    pub fn new(stdin: ChildStdin, stdout: ChildStdout) -> Self {
        StdinTransport { stdin, stdout }
    }

    /// Send a message to the CLI via stdin
    pub async fn send(&mut self, message: &ProtocolMessage) -> Result<()> {
        let json = serde_json::to_string(message)?;
        debug!("Sending to CLI: {}", json);

        self.stdin
            .write_all(format!("{}\n", json).as_bytes())
            .await?;
        self.stdin.flush().await?;

        Ok(())
    }

    /// Read a message from CLI via stdout
    pub async fn receive(&mut self) -> Result<Option<ProtocolMessage>> {
        let mut reader = BufReader::new(&mut self.stdout);
        let mut line = String::new();

        let bytes_read = reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            return Ok(None); // EOF
        }

        debug!("Received from CLI: {}", line.trim());

        let message: ProtocolMessage = serde_json::from_str(&line)?;
        Ok(Some(message))
    }

    /// Close the transport
    pub async fn close(&mut self) -> Result<()> {
        info!("Closing stdin transport");
        self.stdin.shutdown().await?;
        Ok(())
    }
}

/// Create a protocol request message
pub fn create_request(id: u64, method: &str, params: Option<serde_json::Value>) -> ProtocolMessage {
    ProtocolMessage {
        id: Some(id),
        jsonrpc: "2.0".to_string(),
        method: Some(method.to_string()),
        params,
        result: None,
        error: None,
    }
}

/// Create a protocol response message
pub fn create_response(id: u64, result: serde_json::Value) -> ProtocolMessage {
    ProtocolMessage {
        id: Some(id),
        jsonrpc: "2.0".to_string(),
        method: None,
        params: None,
        result: Some(result),
        error: None,
    }
}

/// Create a protocol error message
pub fn create_error(id: u64, code: i64, message: &str) -> ProtocolMessage {
    ProtocolMessage {
        id: Some(id),
        jsonrpc: "2.0".to_string(),
        method: None,
        params: None,
        result: None,
        error: Some(ProtocolError {
            code,
            message: message.to_string(),
            data: None,
        }),
    }
}

/// Create a notification message (no ID required)
pub fn create_notification(method: &str, params: Option<serde_json::Value>) -> ProtocolMessage {
    ProtocolMessage {
        id: None,
        jsonrpc: "2.0".to_string(),
        method: Some(method.to_string()),
        params,
        result: None,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_request() {
        let msg = create_request(1, "initialize", Some(serde_json::json!({"version": "1.0"})));

        assert_eq!(msg.id, Some(1));
        assert_eq!(msg.jsonrpc, "2.0");
        assert_eq!(msg.method, Some("initialize".to_string()));
        assert!(msg.params.is_some());
        assert!(msg.result.is_none());
        assert!(msg.error.is_none());
    }

    #[test]
    fn test_create_response() {
        let msg = create_response(1, serde_json::json!({"status": "ok"}));

        assert_eq!(msg.id, Some(1));
        assert_eq!(msg.jsonrpc, "2.0");
        assert!(msg.method.is_none());
        assert!(msg.result.is_some());
        assert!(msg.error.is_none());
    }

    #[test]
    fn test_create_error() {
        let msg = create_error(1, -32600, "Invalid Request");

        assert_eq!(msg.id, Some(1));
        assert_eq!(msg.jsonrpc, "2.0");
        assert!(msg.method.is_none());
        assert!(msg.result.is_none());
        assert!(msg.error.is_some());

        let error = msg.error.unwrap();
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid Request");
    }

    #[test]
    fn test_create_notification() {
        let msg = create_notification("update", Some(serde_json::json!({"progress": 50})));

        assert_eq!(msg.id, None);
        assert_eq!(msg.jsonrpc, "2.0");
        assert_eq!(msg.method, Some("update".to_string()));
        assert!(msg.params.is_some());
        assert!(msg.result.is_none());
        assert!(msg.error.is_none());
    }

    #[test]
    fn test_protocol_message_serialization() {
        let msg = create_request(42, "test_method", None);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"id\":42"));
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"test_method\""));
    }

    #[test]
    fn test_protocol_message_deserialization() {
        let json = r#"{
            "id": 1,
            "jsonrpc": "2.0",
            "method": "test",
            "params": {"key": "value"}
        }"#;

        let msg: ProtocolMessage = serde_json::from_str(json).unwrap();

        assert_eq!(msg.id, Some(1));
        assert_eq!(msg.jsonrpc, "2.0");
        assert_eq!(msg.method, Some("test".to_string()));
        assert!(msg.params.is_some());
    }

    #[test]
    fn test_protocol_error_structure() {
        let error = ProtocolError {
            code: -32601,
            message: "Method not found".to_string(),
            data: Some(serde_json::json!({"details": "unknown method"})),
        };

        assert_eq!(error.code, -32601);
        assert_eq!(error.message, "Method not found");
        assert!(error.data.is_some());
    }

    #[test]
    fn test_protocol_message_debug_format() {
        let msg = create_request(1, "init", None);
        let debug_str = format!("{:?}", msg);

        assert!(debug_str.contains("ProtocolMessage"));
        assert!(debug_str.contains("jsonrpc"));
    }
}
