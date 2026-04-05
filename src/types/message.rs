use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents a role in the conversation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// A single message content
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MessageContent {
    pub role: MessageRole,
    pub content: String,
}

/// Base SDK message trait
pub trait SDKMessageBase: Clone + Send + Sync + std::fmt::Debug {
    fn session_id(&self) -> &str;
    fn message_type(&self) -> MessageType;
}

/// Types of SDK messages
#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    User,
    Assistant,
    System,
    Result,
    PartialAssistant,
}

/// User message sent to QwenCode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDKUserMessage {
    pub session_id: String,
    pub message: MessageContent,
    pub parent_tool_use_id: Option<String>,
}

impl SDKMessageBase for SDKUserMessage {
    fn session_id(&self) -> &str {
        &self.session_id
    }

    fn message_type(&self) -> MessageType {
        MessageType::User
    }
}

/// Assistant message from QwenCode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDKAssistantMessage {
    pub session_id: String,
    pub message: MessageContent,
}

impl SDKMessageBase for SDKAssistantMessage {
    fn session_id(&self) -> &str {
        &self.session_id
    }

    fn message_type(&self) -> MessageType {
        MessageType::Assistant
    }
}

/// System message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDKSystemMessage {
    pub session_id: String,
    pub message: MessageContent,
}

impl SDKMessageBase for SDKSystemMessage {
    fn session_id(&self) -> &str {
        &self.session_id
    }

    fn message_type(&self) -> MessageType {
        MessageType::System
    }
}

/// Result message when query completes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDKResultMessage {
    pub session_id: String,
    pub result: serde_json::Value,
    pub exit_code: i32,
}

impl SDKMessageBase for SDKResultMessage {
    fn session_id(&self) -> &str {
        &self.session_id
    }

    fn message_type(&self) -> MessageType {
        MessageType::Result
    }
}

/// Partial assistant message for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDKPartialAssistantMessage {
    pub session_id: String,
    pub message: MessageContent,
    pub is_complete: bool,
}

impl SDKMessageBase for SDKPartialAssistantMessage {
    fn session_id(&self) -> &str {
        &self.session_id
    }

    fn message_type(&self) -> MessageType {
        MessageType::PartialAssistant
    }
}

/// Enum wrapping all message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SDKMessage {
    User(SDKUserMessage),
    Assistant(SDKAssistantMessage),
    System(SDKSystemMessage),
    Result(SDKResultMessage),
    PartialAssistant(SDKPartialAssistantMessage),
}

impl SDKMessage {
    pub fn session_id(&self) -> &str {
        match self {
            SDKMessage::User(m) => &m.session_id,
            SDKMessage::Assistant(m) => &m.session_id,
            SDKMessage::System(m) => &m.session_id,
            SDKMessage::Result(m) => &m.session_id,
            SDKMessage::PartialAssistant(m) => &m.session_id,
        }
    }

    pub fn message_type(&self) -> MessageType {
        match self {
            SDKMessage::User(_) => MessageType::User,
            SDKMessage::Assistant(_) => MessageType::Assistant,
            SDKMessage::System(_) => MessageType::System,
            SDKMessage::Result(_) => MessageType::Result,
            SDKMessage::PartialAssistant(_) => MessageType::PartialAssistant,
        }
    }
}

// Type guard functions (idiomatic Rust pattern matching helpers)
impl SDKMessage {
    pub fn is_user_message(&self) -> bool {
        matches!(self, SDKMessage::User(_))
    }

    pub fn is_assistant_message(&self) -> bool {
        matches!(self, SDKMessage::Assistant(_))
    }

    pub fn is_system_message(&self) -> bool {
        matches!(self, SDKMessage::System(_))
    }

    pub fn is_result_message(&self) -> bool {
        matches!(self, SDKMessage::Result(_))
    }

    pub fn is_partial_assistant_message(&self) -> bool {
        matches!(self, SDKMessage::PartialAssistant(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_message_creation() {
        let msg = SDKUserMessage {
            session_id: "test-session".to_string(),
            message: MessageContent {
                role: MessageRole::User,
                content: "Hello".to_string(),
            },
            parent_tool_use_id: None,
        };

        assert_eq!(msg.session_id(), "test-session");
        assert_eq!(msg.message_type(), MessageType::User);

        let wrapped = SDKMessage::User(msg.clone());
        assert!(wrapped.is_user_message());
    }

    #[test]
    fn test_assistant_message_creation() {
        let msg = SDKAssistantMessage {
            session_id: "test-session".to_string(),
            message: MessageContent {
                role: MessageRole::Assistant,
                content: "Hi there!".to_string(),
            },
        };

        assert_eq!(msg.session_id(), "test-session");
        assert_eq!(msg.message_type(), MessageType::Assistant);

        let wrapped = SDKMessage::Assistant(msg.clone());
        assert!(wrapped.is_assistant_message());
    }

    #[test]
    fn test_system_message_creation() {
        let msg = SDKSystemMessage {
            session_id: "test-session".to_string(),
            message: MessageContent {
                role: MessageRole::System,
                content: "System initialized".to_string(),
            },
        };

        assert_eq!(msg.session_id(), "test-session");
        assert_eq!(msg.message_type(), MessageType::System);

        let wrapped = SDKMessage::System(msg.clone());
        assert!(wrapped.is_system_message());
    }

    #[test]
    fn test_result_message_creation() {
        let msg = SDKResultMessage {
            session_id: "test-session".to_string(),
            result: serde_json::json!({"status": "success"}),
            exit_code: 0,
        };

        assert_eq!(msg.session_id(), "test-session");
        assert_eq!(msg.message_type(), MessageType::Result);

        let wrapped = SDKMessage::Result(msg.clone());
        assert!(wrapped.is_result_message());
    }

    #[test]
    fn test_partial_assistant_message_creation() {
        let msg = SDKPartialAssistantMessage {
            session_id: "test-session".to_string(),
            message: MessageContent {
                role: MessageRole::Assistant,
                content: "Partial...".to_string(),
            },
            is_complete: false,
        };

        assert_eq!(msg.session_id(), "test-session");
        assert_eq!(msg.message_type(), MessageType::PartialAssistant);

        let wrapped = SDKMessage::PartialAssistant(msg.clone());
        assert!(wrapped.is_partial_assistant_message());
    }

    #[test]
    fn test_sdk_message_enum_user() {
        let user_msg = SDKUserMessage {
            session_id: "s1".to_string(),
            message: MessageContent {
                role: MessageRole::User,
                content: "test".to_string(),
            },
            parent_tool_use_id: None,
        };

        let msg = SDKMessage::User(user_msg);
        assert!(msg.is_user_message());
        assert_eq!(msg.session_id(), "s1");
    }

    #[test]
    fn test_sdk_message_enum_assistant() {
        let assistant_msg = SDKAssistantMessage {
            session_id: "s2".to_string(),
            message: MessageContent {
                role: MessageRole::Assistant,
                content: "response".to_string(),
            },
        };

        let msg = SDKMessage::Assistant(assistant_msg);
        assert!(msg.is_assistant_message());
        assert_eq!(msg.session_id(), "s2");
    }

    #[test]
    fn test_message_role_serialization() {
        let role = MessageRole::User;
        let serialized = serde_json::to_string(&role).unwrap();
        assert_eq!(serialized, "\"user\"");

        let deserialized: MessageRole = serde_json::from_str(&serialized).unwrap();
        assert!(matches!(deserialized, MessageRole::User));
    }

    #[test]
    fn test_message_content_serialization() {
        let content = MessageContent {
            role: MessageRole::Assistant,
            content: "Hello".to_string(),
        };

        let serialized = serde_json::to_string(&content).unwrap();
        assert!(serialized.contains("\"role\":\"assistant\""));
        assert!(serialized.contains("\"content\":\"Hello\""));
    }

    #[test]
    fn test_type_guards_all_return_false_for_wrong_type() {
        let msg = SDKMessage::User(SDKUserMessage {
            session_id: "s1".to_string(),
            message: MessageContent {
                role: MessageRole::User,
                content: "test".to_string(),
            },
            parent_tool_use_id: None,
        });

        assert!(!msg.is_assistant_message());
        assert!(!msg.is_system_message());
        assert!(!msg.is_result_message());
        assert!(!msg.is_partial_assistant_message());
    }

    #[test]
    fn test_message_debug_format() {
        let msg = SDKUserMessage {
            session_id: "debug-session".to_string(),
            message: MessageContent {
                role: MessageRole::User,
                content: "Debug test".to_string(),
            },
            parent_tool_use_id: None,
        };

        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("debug-session"));
        assert!(debug_str.contains("Debug test"));
    }
}
