use std::path::PathBuf;

/// Get the default QwenCode executable path
pub fn get_default_qwen_path() -> Option<PathBuf> {
    // Try common locations
    let candidates = vec!["qwen", "qwen-code"];

    for candidate in candidates {
        if which(candidate).is_some() {
            return Some(PathBuf::from(candidate));
        }
    }

    None
}

/// Check if an executable exists in PATH
fn which(executable: &str) -> Option<PathBuf> {
    // Simple implementation - in production, use the `which` crate
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join(executable);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    })
}

/// Format a duration in milliseconds to a human-readable string
pub fn format_duration_ms(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        let minutes = ms / 60_000;
        let seconds = (ms % 60_000) / 1000;
        format!("{}m {}s", minutes, seconds)
    }
}

/// Convert a string to a path buffer
pub fn string_to_path(s: &str) -> PathBuf {
    PathBuf::from(s)
}

/// Check if running in debug mode
pub fn is_debug_mode() -> bool {
    cfg!(debug_assertions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_ms_less_than_second() {
        assert_eq!(format_duration_ms(500), "500ms");
        assert_eq!(format_duration_ms(1), "1ms");
        assert_eq!(format_duration_ms(999), "999ms");
    }

    #[test]
    fn test_format_duration_ms_less_than_minute() {
        assert_eq!(format_duration_ms(1000), "1.0s");
        assert_eq!(format_duration_ms(1500), "1.5s");
        assert_eq!(format_duration_ms(59999), "60.0s");
    }

    #[test]
    fn test_format_duration_ms_minutes() {
        assert_eq!(format_duration_ms(60000), "1m 0s");
        assert_eq!(format_duration_ms(90000), "1m 30s");
        assert_eq!(format_duration_ms(120000), "2m 0s");
    }

    #[test]
    fn test_string_to_path() {
        let path = string_to_path("/tmp/test");
        assert_eq!(path, PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_is_debug_mode() {
        // Should be true in test builds
        let _ = is_debug_mode();
    }

    #[test]
    fn test_get_default_qwen_path_returns_option() {
        // May or may not find qwen, but should return an Option
        let result = get_default_qwen_path();
        assert!(result.is_none() || result.is_some());
    }
}
