use crate::types::message::SDKMessage;
use async_channel::{Receiver, Sender};
use tracing::debug;

/// Stream of SDK messages from the CLI
pub struct MessageStream {
    receiver: Receiver<Result<SDKMessage, anyhow::Error>>,
    closed: bool,
}

impl MessageStream {
    pub fn new(receiver: Receiver<Result<SDKMessage, anyhow::Error>>) -> Self {
        MessageStream {
            receiver,
            closed: false,
        }
    }

    /// Receive the next message from the stream
    pub async fn next_message(&self) -> Option<Result<SDKMessage, anyhow::Error>> {
        match self.receiver.recv().await {
            Ok(msg) => {
                debug!("Received message from stream");
                Some(msg)
            }
            Err(_) => {
                debug!("Message stream closed");
                None
            }
        }
    }

    /// Check if the stream is closed
    pub fn is_closed(&self) -> bool {
        self.closed || self.receiver.is_closed()
    }
}

/// Message handler that processes incoming messages
pub struct MessageHandler {
    sender: Sender<Result<SDKMessage, anyhow::Error>>,
}

impl MessageHandler {
    pub fn new(sender: Sender<Result<SDKMessage, anyhow::Error>>) -> Self {
        MessageHandler { sender }
    }

    /// Send a message to the stream
    pub async fn send_message(&self, message: SDKMessage) -> Result<(), anyhow::Error> {
        self.sender.send(Ok(message)).await?;
        Ok(())
    }

    /// Send an error to the stream
    pub async fn send_error(&self, error: anyhow::Error) -> Result<(), anyhow::Error> {
        self.sender.send(Err(error)).await?;
        Ok(())
    }

    /// Close the handler
    pub fn close(&self) {
        self.sender.close();
    }
}

/// Create a message stream pair (sender and MessageStream)
pub fn create_message_stream() -> (MessageHandler, MessageStream) {
    let (sender, receiver) = async_channel::unbounded();
    let handler = MessageHandler::new(sender);
    let stream = MessageStream::new(receiver);

    (handler, stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::message::{MessageContent, MessageRole, SDKUserMessage};

    #[tokio::test]
    async fn test_message_stream_send_and_receive() {
        let (handler, stream) = create_message_stream();

        let message = SDKMessage::User(SDKUserMessage {
            session_id: "test".to_string(),
            message: MessageContent {
                role: MessageRole::User,
                content: "Hello".to_string(),
            },
            parent_tool_use_id: None,
        });

        handler.send_message(message.clone()).await.unwrap();

        let received = stream.next_message().await.unwrap().unwrap();

        assert_eq!(received.session_id(), "test");
        assert!(received.is_user_message());
    }

    #[tokio::test]
    async fn test_message_stream_send_error() {
        let (handler, stream) = create_message_stream();

        let error = anyhow::anyhow!("Test error");
        handler.send_error(error).await.unwrap();

        let received = stream.next_message().await.unwrap();
        assert!(received.is_err());
    }

    #[tokio::test]
    async fn test_message_stream_close() {
        let (handler, stream) = create_message_stream();

        assert!(!stream.is_closed());

        handler.close();
        assert!(stream.is_closed());
    }

    #[tokio::test]
    async fn test_message_stream_multiple_messages() {
        let (handler, stream) = create_message_stream();

        for i in 0..3 {
            let message = SDKMessage::User(SDKUserMessage {
                session_id: format!("session-{}", i),
                message: MessageContent {
                    role: MessageRole::User,
                    content: format!("Message {}", i),
                },
                parent_tool_use_id: None,
            });

            handler.send_message(message).await.unwrap();
        }

        for i in 0..3 {
            let received = stream.next_message().await.unwrap().unwrap();
            assert_eq!(received.session_id(), format!("session-{}", i));
        }
    }

    #[test]
    fn test_message_stream_initial_state() {
        let (handler, stream) = create_message_stream();

        // Stream is not closed while handler is alive
        assert!(!stream.is_closed());

        // Prevent unused variable warning
        drop(handler);
    }

    #[tokio::test]
    async fn test_message_handler_creation() {
        let (handler, stream) = create_message_stream();

        let message = SDKMessage::User(SDKUserMessage {
            session_id: "test".to_string(),
            message: MessageContent {
                role: MessageRole::User,
                content: "test".to_string(),
            },
            parent_tool_use_id: None,
        });

        handler.send_message(message).await.unwrap();
        let received = stream.next_message().await.unwrap().unwrap();
        assert_eq!(received.session_id(), "test");
    }
}
