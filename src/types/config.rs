use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::types::mcp::McpServerConfig;
use crate::types::permission::PermissionMode;

/// Authentication type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    #[default]
    Openai,
    QwenOauth,
}

/// System prompt configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SystemPromptConfig {
    /// Custom system prompt string
    Custom(String),
    /// Preset with optional append text
    Preset {
        preset: String,
        append: Option<String>,
    },
}

/// Timeout configuration for various SDK operations
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(default)]
pub struct TimeoutConfig {
    /// Timeout for can_use_tool callback (milliseconds)
    #[builder(default = "60000")]
    pub can_use_tool: u64,

    /// Timeout for MCP tool requests (milliseconds)
    #[builder(default = "60000")]
    pub mcp_request: u64,

    /// Timeout for control requests (milliseconds)
    #[builder(default = "60000")]
    pub control_request: u64,

    /// Timeout for stream close (milliseconds)
    #[builder(default = "15000")]
    pub stream_close: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        TimeoutConfig {
            can_use_tool: 60000,
            mcp_request: 60000,
            control_request: 60000,
            stream_close: 15000,
        }
    }
}

/// Subagent configuration
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(default)]
pub struct SubagentConfig {
    #[builder(default)]
    pub name: String,
    #[builder(default)]
    pub description: String,
    #[builder(default)]
    pub tools: Option<Vec<String>>,
}

impl Default for SubagentConfig {
    fn default() -> Self {
        SubagentConfig {
            name: String::new(),
            description: String::new(),
            tools: None,
        }
    }
}

/// Query options for configuring SDK behavior
#[derive(Debug, Clone, Builder, Serialize, Deserialize)]
#[builder(default)]
pub struct QueryOptions {
    /// Working directory for the session
    #[builder(default, setter(into))]
    pub cwd: Option<PathBuf>,

    /// AI model to use (e.g., "qwen-max", "qwen-plus")
    #[builder(default, setter(into))]
    pub model: Option<String>,

    /// Path to qwen executable (auto-detected if None)
    #[builder(default, setter(into))]
    pub path_to_qwen_executable: Option<String>,

    /// Permission mode for tool approval
    #[builder(default)]
    pub permission_mode: PermissionMode,

    /// Environment variables merged with current process
    #[builder(default)]
    pub env: Option<HashMap<String, String>>,

    /// System prompt configuration
    #[builder(default)]
    pub system_prompt: Option<SystemPromptConfig>,

    /// MCP servers configuration
    #[builder(default)]
    pub mcp_servers: Option<HashMap<String, McpServerConfig>>,

    /// Enable debug logging
    #[builder(default)]
    pub debug: bool,

    /// Maximum session turns (-1 for unlimited)
    #[builder(default = "-1")]
    pub max_session_turns: i32,

    /// Allowlist of tools
    #[builder(default)]
    pub core_tools: Option<Vec<String>>,

    /// Denylist of tools
    #[builder(default)]
    pub exclude_tools: Option<Vec<String>>,

    /// Tools that bypass can_use_tool and auto-execute
    #[builder(default)]
    pub allowed_tools: Option<Vec<String>>,

    /// Authentication type
    #[builder(default)]
    pub auth_type: AuthType,

    /// Subagent configuration
    #[builder(default)]
    pub agents: Option<Vec<SubagentConfig>>,

    /// Include partial messages during generation
    #[builder(default)]
    pub include_partial_messages: bool,

    /// Session ID to resume history
    #[builder(default, setter(into))]
    pub resume: Option<String>,

    /// Session ID without resuming history
    #[builder(default, setter(into))]
    pub session_id: Option<String>,

    /// Timeout configuration
    #[builder(default)]
    pub timeouts: Option<TimeoutConfig>,
}

impl Default for QueryOptions {
    fn default() -> Self {
        QueryOptions {
            cwd: None,
            model: None,
            path_to_qwen_executable: None,
            permission_mode: PermissionMode::default(),
            env: None,
            system_prompt: None,
            mcp_servers: None,
            debug: false,
            max_session_turns: -1,
            core_tools: None,
            exclude_tools: None,
            allowed_tools: None,
            auth_type: AuthType::default(),
            agents: None,
            include_partial_messages: false,
            resume: None,
            session_id: None,
            timeouts: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_type_default() {
        let auth = AuthType::default();
        assert_eq!(auth, AuthType::Openai);
    }

    #[test]
    fn test_auth_type_qwen_oauth() {
        let auth = AuthType::QwenOauth;
        assert_eq!(auth, AuthType::QwenOauth);
    }

    #[test]
    fn test_auth_type_serialization() {
        let auth = AuthType::Openai;
        let serialized = serde_json::to_string(&auth).unwrap();
        assert_eq!(serialized, "\"openai\"");

        let deserialized: AuthType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, AuthType::Openai);
    }

    #[test]
    fn test_system_prompt_config_custom() {
        let config = SystemPromptConfig::Custom("Custom prompt".to_string());

        match &config {
            SystemPromptConfig::Custom(prompt) => {
                assert_eq!(prompt, "Custom prompt");
            }
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn test_system_prompt_config_preset_without_append() {
        let config = SystemPromptConfig::Preset {
            preset: "qwen_code".to_string(),
            append: None,
        };

        match &config {
            SystemPromptConfig::Preset { preset, append } => {
                assert_eq!(preset, "qwen_code");
                assert!(append.is_none());
            }
            _ => panic!("Expected Preset variant"),
        }
    }

    #[test]
    fn test_system_prompt_config_preset_with_append() {
        let config = SystemPromptConfig::Preset {
            preset: "qwen_code".to_string(),
            append: Some("Additional instructions".to_string()),
        };

        match &config {
            SystemPromptConfig::Preset { preset, append } => {
                assert_eq!(preset, "qwen_code");
                assert_eq!(append, &Some("Additional instructions".to_string()));
            }
            _ => panic!("Expected Preset variant"),
        }
    }

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();

        assert_eq!(config.can_use_tool, 60000);
        assert_eq!(config.mcp_request, 60000);
        assert_eq!(config.control_request, 60000);
        assert_eq!(config.stream_close, 15000);
    }

    #[test]
    fn test_timeout_config_builder() {
        let config = TimeoutConfigBuilder::default()
            .can_use_tool(30000)
            .mcp_request(120000)
            .build()
            .unwrap();

        assert_eq!(config.can_use_tool, 30000);
        assert_eq!(config.mcp_request, 120000);
        assert_eq!(config.control_request, 60000); // default
        assert_eq!(config.stream_close, 15000); // default
    }

    #[test]
    fn test_subagent_config_default() {
        let config = SubagentConfig::default();

        assert_eq!(config.name, "");
        assert_eq!(config.description, "");
        assert!(config.tools.is_none());
    }

    #[test]
    fn test_subagent_config_builder() {
        let config = SubagentConfigBuilder::default()
            .name("test-agent".to_string())
            .description("Test agent".to_string())
            .tools(Some(vec!["tool1".to_string(), "tool2".to_string()]))
            .build()
            .unwrap();

        assert_eq!(config.name, "test-agent");
        assert_eq!(config.description, "Test agent");
        assert_eq!(
            config.tools,
            Some(vec!["tool1".to_string(), "tool2".to_string()])
        );
    }

    #[test]
    fn test_query_options_default() {
        let options = QueryOptions::default();

        assert!(options.cwd.is_none());
        assert!(options.model.is_none());
        assert!(options.path_to_qwen_executable.is_none());
        assert_eq!(options.permission_mode, PermissionMode::default());
        assert!(options.env.is_none());
        assert!(options.system_prompt.is_none());
        assert!(options.mcp_servers.is_none());
        assert!(!options.debug);
        assert_eq!(options.max_session_turns, -1);
        assert!(options.core_tools.is_none());
        assert!(options.exclude_tools.is_none());
        assert!(options.allowed_tools.is_none());
        assert_eq!(options.auth_type, AuthType::default());
        assert!(options.agents.is_none());
        assert!(!options.include_partial_messages);
        assert!(options.resume.is_none());
        assert!(options.session_id.is_none());
        assert!(options.timeouts.is_none());
    }

    #[test]
    fn test_query_options_builder() {
        let options = QueryOptionsBuilder::default()
            .model("qwen-max".to_string())
            .debug(true)
            .max_session_turns(10)
            .include_partial_messages(true)
            .build()
            .unwrap();

        assert_eq!(options.model, Some("qwen-max".to_string()));
        assert!(options.debug);
        assert_eq!(options.max_session_turns, 10);
        assert!(options.include_partial_messages);
        // Check defaults for unset fields
        assert!(options.cwd.is_none());
        assert_eq!(options.permission_mode, PermissionMode::default());
    }

    #[test]
    fn test_query_options_with_cwd_pathbuf() {
        let path = PathBuf::from("/tmp/test");
        let options = QueryOptionsBuilder::default()
            .cwd(path.clone())
            .build()
            .unwrap();

        assert_eq!(options.cwd, Some(path));
    }

    #[test]
    fn test_query_options_serialization() {
        let options = QueryOptions {
            model: Some("qwen-plus".to_string()),
            debug: true,
            max_session_turns: 5,
            ..Default::default()
        };

        let serialized = serde_json::to_string(&options).unwrap();
        assert!(serialized.contains("\"model\":\"qwen-plus\""));
        assert!(serialized.contains("\"debug\":true"));
        assert!(serialized.contains("\"max_session_turns\":5"));
    }

    #[test]
    fn test_timeout_config_all_custom() {
        let config = TimeoutConfigBuilder::default()
            .can_use_tool(10000)
            .mcp_request(20000)
            .control_request(30000)
            .stream_close(5000)
            .build()
            .unwrap();

        assert_eq!(config.can_use_tool, 10000);
        assert_eq!(config.mcp_request, 20000);
        assert_eq!(config.control_request, 30000);
        assert_eq!(config.stream_close, 5000);
    }
}
