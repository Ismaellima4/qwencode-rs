// stdin/stdout communication layer
// This module will handle spawning the QwenCode CLI process
// and managing stdin/stdout communication

use anyhow::Result;
use tokio::process::Command;
use tracing::{debug, info};

/// Spawn the QwenCode CLI process and return stdin/stdout handles
pub async fn spawn_qwen_process(executable_path: Option<&str>) -> Result<tokio::process::Child> {
    let executable = executable_path.unwrap_or("qwen");

    info!("Spawning QwenCode process: {}", executable);

    let mut cmd = Command::new(executable);
    cmd.kill_on_drop(true)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let child = cmd.spawn()?;

    debug!("QwenCode process spawned successfully");
    Ok(child)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_spawn_qwen_process_function_exists() {
        // Just verify the function compiles
        // Actual process spawning will be tested in integration tests
    }
}
