// CLI process communication layer
// Handles spawning the QwenCode CLI process and managing bidirectional communication

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::transport::protocol::{create_notification, create_request, ProtocolMessage};
use crate::types::config::QueryOptions;
use crate::types::message::SDKMessage;

/// Request to send to the CLI process
#[derive(Debug, Clone, Serialize)]
pub struct CLIRequest {
    /// Request type
    #[serde(rename = "type")]
    pub request_type: String,
    /// Prompt text
    pub prompt: String,
    /// Session ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Query options
    #[serde(flatten)]
    pub options: QueryOptions,
}

/// Initialize request
#[derive(Debug, Clone, Serialize)]
pub struct InitializeRequest {
    pub protocol_version: String,
    pub client: String,
    pub client_version: String,
}

/// Initialize response from CLI
#[derive(Debug, Clone, Deserialize)]
pub struct InitializeResponse {
    pub protocol_version: String,
    pub capabilities: CLICapabilities,
}

/// CLI capabilities
#[derive(Debug, Clone, Deserialize)]
pub struct CLICapabilities {
    #[serde(default)]
    pub streaming: bool,
    #[serde(default)]
    pub tool_use: bool,
    #[serde(default)]
    pub multi_turn: bool,
}

/// Spawn QwenCode CLI process and return stdin/stdout handles
pub async fn spawn_cli_process(executable_path: Option<&str>) -> Result<CLIProcess> {
    let executable = executable_path.unwrap_or("qwen");

    info!("Spawning QwenCode CLI process: {}", executable);

    let mut child = tokio::process::Command::new(executable)
        .kill_on_drop(true)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn QwenCode CLI process")?;

    let stdin = child.stdin.take().context("Failed to get stdin handle")?;

    let stdout = child.stdout.take().context("Failed to get stdout handle")?;

    let stderr = child.stderr.take().context("Failed to get stderr handle")?;

    // Spawn stderr reader task
    let (stderr_tx, stderr_rx) = mpsc::unbounded_channel::<String>();
    tokio::spawn(read_stderr(stderr, stderr_tx));

    debug!(
        "QwenCode CLI process spawned successfully (PID: {:?})",
        child.id()
    );

    Ok(CLIProcess {
        child,
        stdin,
        stdout,
        stderr_rx,
        message_counter: 0,
    })
}

/// Handle to a spawned CLI process
pub struct CLIProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
    stderr_rx: mpsc::UnboundedReceiver<String>,
    message_counter: u64,
}

impl CLIProcess {
    /// Send initialize request and wait for response
    pub async fn initialize(
        &mut self,
        cancel_token: &CancellationToken,
    ) -> Result<InitializeResponse> {
        info!("Initializing CLI connection");

        let init_request = InitializeRequest {
            protocol_version: "1.0".to_string(),
            client: "qwencode-rs".to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let json = serde_json::to_string(&init_request)?;
        let message = format!("{}\n", json);

        self.stdin
            .write_all(message.as_bytes())
            .await
            .context("Failed to send initialize request")?;
        self.stdin.flush().await.context("Failed to flush stdin")?;

        debug!("Initialize request sent");

        // Read response
        let mut reader = BufReader::new(&mut self.stdout);
        let mut line = String::new();

        tokio::select! {
            result = reader.read_line(&mut line) => {
                let bytes_read = result.context("Failed to read initialize response")?;
                if bytes_read == 0 {
                    return Err(anyhow::anyhow!("CLI process exited before responding"));
                }

                debug!("Initialize response: {}", line.trim());
                let response: InitializeResponse = serde_json::from_str(&line)
                    .context("Failed to parse initialize response")?;

                info!("CLI initialized with protocol version: {}", response.protocol_version);
                Ok(response)
            }
            _ = cancel_token.cancelled() => {
                Err(anyhow::anyhow!("Initialize cancelled"))
            }
        }
    }

    /// Send a query request to the CLI
    pub async fn send_query(&mut self, request: &CLIRequest) -> Result<()> {
        self.message_counter += 1;
        let id = self.message_counter;

        let params = serde_json::to_value(request)?;
        let message = create_request(id, "query", Some(params));

        self.send_message(&message).await
    }

    /// Send a generic protocol message
    async fn send_message(&mut self, message: &ProtocolMessage) -> Result<()> {
        let json = serde_json::to_string(message)?;
        let line = format!("{}\n", json);

        debug!("Sending to CLI: {}", json);

        self.stdin
            .write_all(line.as_bytes())
            .await
            .context("Failed to write to stdin")?;
        self.stdin.flush().await.context("Failed to flush stdin")?;

        Ok(())
    }

    /// Read next message from stdout
    pub async fn read_message(&mut self) -> Result<Option<ProtocolMessage>> {
        let mut reader = BufReader::new(&mut self.stdout);
        let mut line = String::new();

        let bytes_read = reader
            .read_line(&mut line)
            .await
            .context("Failed to read from stdout")?;

        if bytes_read == 0 {
            debug!("stdout closed (EOF)");
            return Ok(None);
        }

        let line = line.trim().to_string();
        if line.is_empty() {
            return Ok(None);
        }

        debug!("Received from CLI: {}", line);

        let message: ProtocolMessage = serde_json::from_str(&line)
            .with_context(|| format!("Failed to parse message: {}", line))?;

        Ok(Some(message))
    }

    /// Check if process is still running
    pub fn is_running(&mut self) -> bool {
        self.child
            .try_wait()
            .map(|opt| opt.is_none())
            .unwrap_or(false)
    }

    /// Get process ID
    pub fn pid(&self) -> Option<u32> {
        self.child.id()
    }

    /// Gracefully shutdown the process
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down CLI process (PID: {:?})", self.pid());

        // Send close notification
        let close_msg = create_notification("close", None);
        let _ = self.send_message(&close_msg).await;

        // Wait a bit for graceful shutdown
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Try graceful exit
        if let Ok(Some(status)) = self.child.try_wait() {
            debug!("Process exited with status: {:?}", status);
            return Ok(());
        }

        // Force kill if still running
        if let Err(e) = self.child.kill().await {
            warn!("Failed to kill process: {}", e);
        }

        match self.child.wait().await {
            Ok(status) => {
                info!("Process terminated with status: {:?}", status);
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to wait for process: {}", e)),
        }
    }

    /// Poll stderr for any messages
    pub fn try_receive_stderr(&mut self) -> Option<String> {
        self.stderr_rx.try_recv().ok()
    }
}

/// Read from stderr and send to channel
async fn read_stderr(stderr: tokio::process::ChildStderr, sender: mpsc::UnboundedSender<String>) {
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();

    loop {
        match reader.read_line(&mut line).await {
            Ok(0) => {
                debug!("stderr closed");
                break;
            }
            Ok(_) => {
                let trimmed = line.trim().to_string();
                if !trimmed.is_empty() {
                    debug!("stderr: {}", trimmed);
                    let _ = sender.send(trimmed);
                }
                line.clear();
            }
            Err(e) => {
                error!("Error reading stderr: {}", e);
                break;
            }
        }
    }
}

/// Convert a ProtocolMessage to SDKMessage
pub fn protocol_to_sdk_message(message: &ProtocolMessage) -> Result<Option<SDKMessage>> {
    // Check if it's a method call (incoming message from CLI)
    if let Some(method) = &message.method {
        match method.as_str() {
            "assistant_message" => {
                if let Some(params) = &message.params {
                    let content = params
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    return Ok(Some(SDKMessage::from_assistant_text(&content)));
                }
            }
            "result" => {
                if let Some(params) = &message.params {
                    return Ok(Some(SDKMessage::from_result_value(params.clone())));
                }
            }
            "error" => {
                if let Some(error) = &message.error {
                    return Err(anyhow::anyhow!("CLI error: {}", error.message));
                }
            }
            _ => {
                debug!("Unknown method: {}", method);
            }
        }
    }

    // Check if it's a response
    if let Some(result) = &message.result {
        return Ok(Some(SDKMessage::from_result_value(result.clone())));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_request_serialization() {
        let request = CLIRequest {
            request_type: "query".to_string(),
            prompt: "Hello".to_string(),
            session_id: Some("test-session".to_string()),
            options: QueryOptions::default(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"type\":\"query\""));
        assert!(json.contains("\"prompt\":\"Hello\""));
        assert!(json.contains("\"session_id\":\"test-session\""));
    }

    #[test]
    fn test_initialize_request_structure() {
        let request = InitializeRequest {
            protocol_version: "1.0".to_string(),
            client: "qwencode-rs".to_string(),
            client_version: "0.1.0".to_string(),
        };

        assert_eq!(request.protocol_version, "1.0");
        assert_eq!(request.client, "qwencode-rs");
        assert_eq!(request.client_version, "0.1.0");
    }

    #[test]
    fn test_protocol_to_sdk_message_assistant() {
        let protocol_msg = ProtocolMessage {
            id: Some(1),
            jsonrpc: "2.0".to_string(),
            method: Some("assistant_message".to_string()),
            params: Some(serde_json::json!({
                "content": "Hello from assistant"
            })),
            result: None,
            error: None,
        };

        let sdk_msg = protocol_to_sdk_message(&protocol_msg).unwrap().unwrap();
        assert!(sdk_msg.is_assistant_message());
    }

    #[test]
    fn test_protocol_to_sdk_message_result() {
        let protocol_msg = ProtocolMessage {
            id: Some(2),
            jsonrpc: "2.0".to_string(),
            method: None,
            params: None,
            result: Some(serde_json::json!({
                "status": "success",
                "data": "test data"
            })),
            error: None,
        };

        let sdk_msg = protocol_to_sdk_message(&protocol_msg).unwrap().unwrap();
        assert!(sdk_msg.is_result_message());
    }

    #[test]
    fn test_protocol_to_sdk_message_error() {
        let protocol_msg = ProtocolMessage {
            id: Some(3),
            jsonrpc: "2.0".to_string(),
            method: Some("error".to_string()),
            params: None,
            result: None,
            error: Some(crate::transport::protocol::ProtocolError {
                code: -1,
                message: "Something went wrong".to_string(),
                data: None,
            }),
        };

        let result = protocol_to_sdk_message(&protocol_msg);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("CLI error"));
    }

    #[test]
    fn test_protocol_to_sdk_message_unknown() {
        let protocol_msg = ProtocolMessage {
            id: Some(4),
            jsonrpc: "2.0".to_string(),
            method: Some("unknown_method".to_string()),
            params: None,
            result: None,
            error: None,
        };

        let result = protocol_to_sdk_message(&protocol_msg).unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cli_request_with_options() {
        let options = QueryOptions {
            model: Some("qwen-max".to_string()),
            debug: true,
            ..Default::default()
        };

        let request = CLIRequest {
            request_type: "query".to_string(),
            prompt: "Test prompt".to_string(),
            session_id: None,
            options,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"qwen-max\""));
        assert!(json.contains("\"debug\":true"));
    }
}
