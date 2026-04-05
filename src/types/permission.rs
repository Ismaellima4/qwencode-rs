use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

/// Permission mode for tool execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PermissionMode {
    /// Read tools execute automatically; write tools require approval
    #[default]
    Default,
    /// Blocks write tools; AI only generates a plan
    Plan,
    /// Automatically approves edit and write_file tools
    AutoEdit,
    /// Automatically approves all tools
    Yolo,
}

/// Result of tool permission check
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "behavior", rename_all = "snake_case")]
pub enum ToolPermissionResult {
    /// Allow tool execution with optional updated input
    Allow {
        #[serde(rename = "updatedInput")]
        updated_input: serde_json::Value,
    },
    /// Deny tool execution with message
    Deny { message: String },
}

/// Callback type for custom tool permission handling
/// Equivalent to TypeScript's CanUseTool type
pub type CanUseToolCallback = Box<
    dyn Fn(
            String,            // tool_name
            serde_json::Value, // input
        )
            -> Pin<Box<dyn Future<Output = Result<ToolPermissionResult, anyhow::Error>> + Send>>
        + Send
        + Sync,
>;

/// Priority chain for permission handling
/// Order: exclude_tools/deny > ask > plan > yolo > allowed_tools/allow > can_use_tool > default behavior
#[derive(Debug, Clone)]
pub struct PermissionChain {
    pub exclude_tools: Vec<String>,
    pub allowed_tools: Vec<String>,
    pub mode: PermissionMode,
}

impl PermissionChain {
    pub fn new(
        exclude_tools: Vec<String>,
        allowed_tools: Vec<String>,
        mode: PermissionMode,
    ) -> Self {
        PermissionChain {
            exclude_tools,
            allowed_tools,
            mode,
        }
    }

    /// Check if a tool is excluded
    pub fn is_excluded(&self, tool_name: &str) -> bool {
        self.exclude_tools
            .iter()
            .any(|t| t == tool_name || t == "*")
    }

    /// Check if a tool is explicitly allowed
    pub fn is_explicitly_allowed(&self, tool_name: &str) -> bool {
        self.allowed_tools.iter().any(|t| t == tool_name)
    }

    /// Check if tool should auto-execute based on mode
    pub fn should_auto_execute(&self, tool_name: &str) -> bool {
        match self.mode {
            PermissionMode::Yolo => true,
            PermissionMode::AutoEdit => tool_name == "edit" || tool_name == "write_file",
            PermissionMode::Plan => false,
            PermissionMode::Default => tool_name.starts_with("read_"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_mode_default() {
        let mode = PermissionMode::default();
        assert_eq!(mode, PermissionMode::Default);
    }

    #[test]
    fn test_permission_mode_serialization() {
        let mode = PermissionMode::Yolo;
        let serialized = serde_json::to_string(&mode).unwrap();
        assert_eq!(serialized, "\"yolo\"");

        let deserialized: PermissionMode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, PermissionMode::Yolo);
    }

    #[test]
    fn test_permission_mode_all_variants() {
        let modes = vec![
            PermissionMode::Default,
            PermissionMode::Plan,
            PermissionMode::AutoEdit,
            PermissionMode::Yolo,
        ];

        for mode in modes {
            let serialized = serde_json::to_string(&mode).unwrap();
            let deserialized: PermissionMode = serde_json::from_str(&serialized).unwrap();
            assert_eq!(mode, deserialized);
        }
    }

    #[test]
    fn test_tool_permission_result_allow() {
        let result = ToolPermissionResult::Allow {
            updated_input: serde_json::json!({"key": "value"}),
        };

        let serialized = serde_json::to_string(&result).unwrap();
        assert!(serialized.contains("\"behavior\":\"allow\""));
        assert!(serialized.contains("\"key\":\"value\""));
    }

    #[test]
    fn test_tool_permission_result_deny() {
        let result = ToolPermissionResult::Deny {
            message: "Access denied".to_string(),
        };

        let serialized = serde_json::to_string(&result).unwrap();
        assert!(serialized.contains("\"behavior\":\"deny\""));
        assert!(serialized.contains("\"message\":\"Access denied\""));
    }

    #[test]
    fn test_permission_chain_is_excluded() {
        let chain = PermissionChain::new(
            vec!["dangerous_tool".to_string(), "write".to_string()],
            vec!["safe_tool".to_string()],
            PermissionMode::Default,
        );

        assert!(chain.is_excluded("dangerous_tool"));
        assert!(chain.is_excluded("write"));
        assert!(!chain.is_excluded("read_file"));
    }

    #[test]
    fn test_permission_chain_is_excluded_wildcard() {
        let chain = PermissionChain::new(vec!["*".to_string()], vec![], PermissionMode::Default);

        assert!(chain.is_excluded("any_tool"));
        assert!(chain.is_excluded("another_tool"));
    }

    #[test]
    fn test_permission_chain_is_explicitly_allowed() {
        let chain = PermissionChain::new(
            vec![],
            vec!["safe_tool".to_string(), "helper".to_string()],
            PermissionMode::Default,
        );

        assert!(chain.is_explicitly_allowed("safe_tool"));
        assert!(chain.is_explicitly_allowed("helper"));
        assert!(!chain.is_explicitly_allowed("unknown_tool"));
    }

    #[test]
    fn test_permission_chain_should_auto_execute_yolo() {
        let chain = PermissionChain::new(vec![], vec![], PermissionMode::Yolo);

        assert!(chain.should_auto_execute("any_tool"));
        assert!(chain.should_auto_execute("write_file"));
    }

    #[test]
    fn test_permission_chain_should_auto_execute_auto_edit() {
        let chain = PermissionChain::new(vec![], vec![], PermissionMode::AutoEdit);

        assert!(chain.should_auto_execute("edit"));
        assert!(chain.should_auto_execute("write_file"));
        assert!(!chain.should_auto_execute("read_file"));
        assert!(!chain.should_auto_execute("bash"));
    }

    #[test]
    fn test_permission_chain_should_auto_execute_plan() {
        let chain = PermissionChain::new(vec![], vec![], PermissionMode::Plan);

        assert!(!chain.should_auto_execute("any_tool"));
    }

    #[test]
    fn test_permission_chain_should_auto_execute_default() {
        let chain = PermissionChain::new(vec![], vec![], PermissionMode::Default);

        assert!(chain.should_auto_execute("read_file"));
        assert!(chain.should_auto_execute("read_directory"));
        assert!(!chain.should_auto_execute("write_file"));
        assert!(!chain.should_auto_execute("edit"));
    }

    #[test]
    fn test_permission_chain_new_builder() {
        let chain = PermissionChain::new(
            vec!["excluded".to_string()],
            vec!["allowed".to_string()],
            PermissionMode::Yolo,
        );

        assert_eq!(chain.exclude_tools, vec!["excluded"]);
        assert_eq!(chain.allowed_tools, vec!["allowed"]);
        assert_eq!(chain.mode, PermissionMode::Yolo);
    }
}
