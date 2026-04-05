use crate::types::message::SDKMessage;
use async_channel::{Receiver, Sender};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::Stream;
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

    /// Check if the stream is closed
    pub fn is_closed(&self) -> bool {
        self.closed || self.receiver.is_closed()
    }
}

impl Stream for MessageStream {
    type Item = Result<SDKMessage, anyhow::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        match Pin::new(&mut this.receiver).poll_recv(cx) {
            Poll::Ready(Some(msg)) => {
                debug!("Received message from stream");
                Poll::Ready(Some(msg))
            }
            Poll::Ready(None) => {
                debug!("Message stream closed");
                this.closed = true;
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
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
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_message_stream_send_and_receive() {
        let (handler, mut stream) = create_message_stream();

        let message = SDKMessage::User(SDKUserMessage {
            session_id: "test".to_string(),
            message: MessageContent {
                role: MessageRole::User,
                content: "Hello".to_string(),
            },
            parent_tool_use_id: None,
        });

        handler.send_message(message.clone()).await.unwrap();

        let received = stream.next().await.unwrap().unwrap();

        assert_eq!(received.session_id(), "test");
        assert!(received.is_user_message());
    }

    #[tokio::test]
    async fn test_message_stream_send_error() {
        let (handler, mut stream) = create_message_stream();

        let error = anyhow::anyhow!("Test error");
        handler.send_error(error).await.unwrap();

        let received = stream.next().await.unwrap();
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
        let (handler, mut stream) = create_message_stream();

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
            let received = stream.next().await.unwrap().unwrap();
            assert_eq!(received.session_id(), format!("session-{}", i));
        }
    }

    #[test]
    fn test_message_stream_initial_state() {
        let (_, stream) = create_message_stream();

        assert!(!stream.is_closed());
    }

    #[tokio::test]
    async fn test_message_handler_creation() {
        let (sender, _) = async_channel::unbounded();
        let handler = MessageHandler::new(sender);

        let message = SDKMessage::User(SDKUserMessage {
            session_id: "test".to_string(),
            message: MessageContent {
                role: MessageRole::User,
                content: "test".to_string(),
            },
            parent_tool_use_id: None,
        });

        handler.send_message(message).await.unwrap();
    }
}
