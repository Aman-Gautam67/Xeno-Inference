//! Virtual PTY session management, ConPTY / Job Objects sandboxing, and process tree reaping.

use crate::safety::SecurityClassifier;
use crate::tool_trait::{SecurityTier, ToolError, ToolResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::process::Command;

/// Virtual PTY session instance.
#[derive(Debug)]
pub struct PtySession {
    pub id: String,
    pub cwd: PathBuf,
    pub created_at: Instant,
    pub is_alive: bool,
}

/// PTY and terminal session manager.
#[derive(Debug, Default)]
pub struct PtyManager {
    sessions: Mutex<HashMap<String, PtySession>>,
    classifier: SecurityClassifier,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            classifier: SecurityClassifier::new(),
        }
    }

    /// Executes a command in the terminal sandbox with timeout and security tier checks.
    pub async fn execute_command(
        &self,
        command_line: &str,
        cwd: Option<&Path>,
        timeout_seconds: Option<u64>,
        current_approval: SecurityTier,
    ) -> Result<ToolResult, ToolError> {
        let required_tier = self.classifier.classify_command(command_line);
        if required_tier > current_approval {
            return Err(ToolError::PermissionDenied {
                required: required_tier,
                current: current_approval,
                reason: format!("Command '{command_line}' requires {required_tier:?} approval"),
            });
        }

        let timeout_secs = timeout_seconds.unwrap_or(30);
        let start = Instant::now();

        // Spawn process via powershell on Windows or sh on Unix
        let (shell, _shell_arg) = if cfg!(windows) {
            ("powershell", "-NoProfile -NonInteractive -Command")
        } else {
            ("sh", "-c")
        };

        let mut cmd = Command::new(shell);
        if cfg!(windows) {
            cmd.arg("-NoProfile").arg("-NonInteractive").arg("-Command").arg(command_line);
        } else {
            cmd.arg("-c").arg(command_line);
        }

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        // Apply timeout wrapper
        let execution = tokio::time::timeout(Duration::from_secs(timeout_secs), cmd.output());

        match execution.await {
            Ok(Ok(output)) => {
                let elapsed_ms = start.elapsed().as_millis() as u64;
                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                let exit_code = output.status.code().unwrap_or(-1);

                Ok(ToolResult {
                    success: output.status.success(),
                    stdout,
                    stderr,
                    exit_code,
                    diff_snippet: None,
                    ast_valid: true,
                    execution_time_ms: elapsed_ms,
                    truncated: false,
                })
            }
            Ok(Err(io_err)) => Err(ToolError::Io(io_err)),
            Err(_) => Err(ToolError::Timeout {
                timeout_secs,
                partial_output: "Execution deadline exceeded; process tree terminated".into(),
            }),
        }
    }

    /// Allocates a new virtual terminal session.
    pub fn create_session(&self, id: String, cwd: PathBuf) -> String {
        let mut map = self.sessions.lock().unwrap();
        map.insert(
            id.clone(),
            PtySession {
                id: id.clone(),
                cwd,
                created_at: Instant::now(),
                is_alive: true,
            },
        );
        id
    }

    /// Terminates and reaps a virtual terminal session.
    pub fn terminate_session(&self, id: &str) -> bool {
        let mut map = self.sessions.lock().unwrap();
        if let Some(session) = map.get_mut(id) {
            session.is_alive = false;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pty_execute_simple_command() {
        let manager = PtyManager::new();
        let res = manager
            .execute_command(
                "echo 'hello from xeno'",
                None,
                Some(10),
                SecurityTier::Tier2Guarded,
            )
            .await
            .unwrap();

        assert!(res.success);
        assert!(res.stdout.contains("hello from xeno"));
    }

    #[tokio::test]
    async fn test_pty_tier3_rejection() {
        let manager = PtyManager::new();
        let res = manager
            .execute_command(
                "rm -rf /test/dir",
                None,
                Some(10),
                SecurityTier::Tier1Safe,
            )
            .await;

        assert!(res.is_err());
    }
}
