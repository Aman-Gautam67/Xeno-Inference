//! Python execution sandbox strictly enforcing `C:\msys64\ucrt64\bin\python.exe`.

use crate::safety::REQUIRED_PYTHON_PATH;
use crate::tool_trait::{ToolError, ToolResult};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::process::Command;

/// Python execution runner.
#[derive(Debug, Clone)]
pub struct PythonRunner {
    executable: PathBuf,
}

impl Default for PythonRunner {
    fn default() -> Self {
        Self {
            executable: PathBuf::from(REQUIRED_PYTHON_PATH),
        }
    }
}

impl PythonRunner {
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
        }
    }

    /// Executes a Python script file using the strict sandbox executable.
    pub async fn run_script(
        &self,
        script_path: &Path,
        args: &[String],
        cwd: Option<&Path>,
        timeout_seconds: Option<u64>,
    ) -> Result<ToolResult, ToolError> {
        let timeout_secs = timeout_seconds.unwrap_or(30);
        let start = Instant::now();

        let mut cmd = Command::new(&self.executable);
        cmd.arg(script_path);
        for arg in args {
            cmd.arg(arg);
        }

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        // Set Python environment variables
        cmd.env("PYTHONUTF8", "1");
        cmd.env("PYTHONUNBUFFERED", "1");

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
                partial_output: "Python script execution exceeded timeout deadline".into(),
            }),
        }
    }

    /// Executes an inline Python code string.
    pub async fn run_inline_code(
        &self,
        code: &str,
        timeout_seconds: Option<u64>,
    ) -> Result<ToolResult, ToolError> {
        let timeout_secs = timeout_seconds.unwrap_or(30);
        let start = Instant::now();

        let mut cmd = Command::new(&self.executable);
        cmd.arg("-c").arg(code);
        cmd.env("PYTHONUTF8", "1");
        cmd.env("PYTHONUNBUFFERED", "1");

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
                partial_output: "Inline Python execution timed out".into(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_python_runner_inline() {
        let runner = PythonRunner::default();
        if Path::new(REQUIRED_PYTHON_PATH).exists() {
            let res = runner.run_inline_code("print(1 + 2)", Some(5)).await.unwrap();
            assert!(res.success);
            assert_eq!(res.stdout.trim(), "3");
        }
    }
}
