/// Validate a model name
pub fn validate_model_name(model: &str) -> bool {
    // Basic validation: non-empty and contains only valid characters
    if model.is_empty() {
        return false;
    }

    // Model names should be reasonable ASCII strings
    model
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '/')
}

/// Validate a session ID
pub fn validate_session_id(session_id: &str) -> bool {
    // Session IDs should be non-empty and reasonable length
    if session_id.is_empty() || session_id.len() > 256 {
        return false;
    }

    // Should contain only valid characters
    session_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Validate a file path
pub fn validate_path(path: &str) -> bool {
    // Basic path validation
    if path.is_empty() {
        return false;
    }

    // Should not contain null bytes
    !path.contains('\0')
}

/// Sanitize a string for safe use
pub fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_ascii_graphic() || c.is_whitespace())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_model_name_valid() {
        assert!(validate_model_name("qwen-max"));
        assert!(validate_model_name("qwen-plus"));
        assert!(validate_model_name("model_123"));
        assert!(validate_model_name("org/model-v1"));
    }

    #[test]
    fn test_validate_model_name_empty() {
        assert!(!validate_model_name(""));
    }

    #[test]
    fn test_validate_model_name_invalid_chars() {
        assert!(!validate_model_name("model name"));
        assert!(!validate_model_name("model@name"));
        assert!(!validate_model_name("model/name!"));
    }

    #[test]
    fn test_validate_session_id_valid() {
        assert!(validate_session_id("session-123"));
        assert!(validate_session_id("abc_def"));
        assert!(validate_session_id("session123"));
    }

    #[test]
    fn test_validate_session_id_empty() {
        assert!(!validate_session_id(""));
    }

    #[test]
    fn test_validate_session_id_too_long() {
        let long_id = "a".repeat(257);
        assert!(!validate_session_id(&long_id));
    }

    #[test]
    fn test_validate_session_id_max_length() {
        let max_id = "a".repeat(256);
        assert!(validate_session_id(&max_id));
    }

    #[test]
    fn test_validate_session_id_invalid_chars() {
        assert!(!validate_session_id("session@id"));
        assert!(!validate_session_id("session/id"));
        assert!(!validate_session_id("session id"));
    }

    #[test]
    fn test_validate_path_valid() {
        assert!(validate_path("/tmp/test"));
        assert!(validate_path("./relative"));
        assert!(validate_path("C:\\Windows\\path"));
    }

    #[test]
    fn test_validate_path_empty() {
        assert!(!validate_path(""));
    }

    #[test]
    fn test_validate_path_with_null() {
        assert!(!validate_path("/tmp/test\0malicious"));
    }

    #[test]
    fn test_sanitize_string_removes_non_ascii() {
        let input = "Hello\x00World\x01Test";
        let sanitized = sanitize_string(input);
        assert_eq!(sanitized, "HelloWorldTest");
    }

    #[test]
    fn test_sanitize_string_keeps_valid_chars() {
        let input = "Hello World 123!@#";
        let sanitized = sanitize_string(input);
        assert_eq!(sanitized, "Hello World 123!@#");
    }

    #[test]
    fn test_sanitize_string_empty() {
        let sanitized = sanitize_string("");
        assert_eq!(sanitized, "");
    }

    #[test]
    fn test_sanitize_string_only_invalid_chars() {
        let input = "\x00\x01\x02\x03";
        let sanitized = sanitize_string(input);
        assert_eq!(sanitized, "");
    }
}
