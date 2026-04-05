use crate::query::session::QueryHandle;
use crate::types::config::QueryOptions;
use anyhow::Result;

/// Query builder for fluent API
pub struct QueryBuilder {
    options: QueryOptions,
    pub prompt: Option<String>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        QueryBuilder {
            options: QueryOptions::default(),
            prompt: None,
        }
    }

    /// Set the prompt for the query
    pub fn prompt(mut self, prompt: &str) -> Self {
        self.prompt = Some(prompt.to_string());
        self
    }

    /// Set the working directory
    pub fn cwd<P: Into<std::path::PathBuf>>(mut self, path: P) -> Self {
        self.options.cwd = Some(path.into());
        self
    }

    /// Set the AI model
    pub fn model(mut self, model: &str) -> Self {
        self.options.model = Some(model.to_string());
        self
    }

    /// Set the permission mode
    pub fn permission_mode(mut self, mode: crate::types::permission::PermissionMode) -> Self {
        self.options.permission_mode = mode;
        self
    }

    /// Enable debug mode
    pub fn debug(mut self, debug: bool) -> Self {
        self.options.debug = debug;
        self
    }

    /// Set maximum session turns
    pub fn max_turns(mut self, max: i32) -> Self {
        self.options.max_session_turns = max;
        self
    }

    /// Set the session ID
    pub fn session_id(mut self, session_id: &str) -> Self {
        self.options.session_id = Some(session_id.to_string());
        self
    }

    /// Build the query options
    pub fn build(self) -> Result<(Option<String>, QueryOptions)> {
        Ok((self.prompt, self.options))
    }

    /// Execute the query (placeholder for now)
    pub async fn execute(self) -> Result<QueryHandle> {
        let (prompt, options) = self.build()?;

        if let Some(ref p) = prompt {
            tracing::info!("Executing query with prompt: {}", p);
        }

        // For now, just create a session handle
        // In the full implementation, this would spawn the CLI process
        let handle = QueryHandle::new(options.session_id.clone());
        Ok(handle)
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        QueryBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::permission::PermissionMode;

    #[test]
    fn test_query_builder_default() {
        let builder = QueryBuilder::new();
        assert!(builder.prompt.is_none());
    }

    #[test]
    fn test_query_builder_prompt() {
        let builder = QueryBuilder::new().prompt("Hello, world!");

        assert_eq!(builder.prompt, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_query_builder_cwd() {
        let builder = QueryBuilder::new().cwd("/tmp/test");

        assert_eq!(
            builder.options.cwd,
            Some(std::path::PathBuf::from("/tmp/test"))
        );
    }

    #[test]
    fn test_query_builder_model() {
        let builder = QueryBuilder::new().model("qwen-max");

        assert_eq!(builder.options.model, Some("qwen-max".to_string()));
    }

    #[test]
    fn test_query_builder_permission_mode() {
        let builder = QueryBuilder::new().permission_mode(PermissionMode::Yolo);

        assert_eq!(builder.options.permission_mode, PermissionMode::Yolo);
    }

    #[test]
    fn test_query_builder_debug() {
        let builder = QueryBuilder::new().debug(true);

        assert!(builder.options.debug);
    }

    #[test]
    fn test_query_builder_max_turns() {
        let builder = QueryBuilder::new().max_turns(10);

        assert_eq!(builder.options.max_session_turns, 10);
    }

    #[test]
    fn test_query_builder_session_id() {
        let builder = QueryBuilder::new().session_id("my-session");

        assert_eq!(builder.options.session_id, Some("my-session".to_string()));
    }

    #[test]
    fn test_query_builder_build() {
        let (prompt, options) = QueryBuilder::new()
            .prompt("Test")
            .model("qwen-plus")
            .debug(true)
            .build()
            .unwrap();

        assert_eq!(prompt, Some("Test".to_string()));
        assert_eq!(options.model, Some("qwen-plus".to_string()));
        assert!(options.debug);
    }

    #[tokio::test]
    async fn test_query_builder_execute() {
        let handle = QueryBuilder::new()
            .prompt("Test query")
            .session_id("test-session")
            .execute()
            .await
            .unwrap();

        assert_eq!(handle.session_id(), "test-session");
        assert!(!handle.is_closed());
    }

    #[test]
    fn test_query_builder_chained_calls() {
        let builder = QueryBuilder::new()
            .prompt("Hello")
            .model("qwen-max")
            .cwd("/tmp")
            .permission_mode(PermissionMode::AutoEdit)
            .debug(true)
            .max_turns(5)
            .session_id("chained-test");

        assert_eq!(builder.prompt, Some("Hello".to_string()));
        assert_eq!(builder.options.model, Some("qwen-max".to_string()));
        assert_eq!(builder.options.cwd, Some(std::path::PathBuf::from("/tmp")));
        assert_eq!(builder.options.permission_mode, PermissionMode::AutoEdit);
        assert!(builder.options.debug);
        assert_eq!(builder.options.max_session_turns, 5);
        assert_eq!(builder.options.session_id, Some("chained-test".to_string()));
    }
}
