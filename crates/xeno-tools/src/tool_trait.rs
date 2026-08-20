//! Standardized Tool trait and execution context contracts.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use thiserror::Error;

/// Security tier classifying tool execution risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SecurityTier {
    /// Tier 1: Safe read-only inspection (auto-approved).
    #[default]
    Tier1Safe = 1,
    /// Tier 2: Guarded mutation with diff snapshot & rollback capability.
    Tier2Guarded = 2,
    /// Tier 3: Destructive or elevated operation requiring explicit user confirmation.
    Tier3Destructive = 3,
}

/// Description and JSON schema of a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub tier: SecurityTier,
}

/// Standardized tool observation execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub diff_snippet: Option<String>,
    pub ast_valid: bool,
    pub execution_time_ms: u64,
    pub truncated: bool,
}

impl ToolResult {
    /// Creates a successful tool result.
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            success: true,
            stdout: stdout.into(),
            stderr: String::new(),
            exit_code: 0,
            diff_snippet: None,
            ast_valid: true,
            execution_time_ms: 0,
            truncated: false,
        }
    }

    /// Creates a failure tool result.
    pub fn failure(exit_code: i32, stderr: impl Into<String>) -> Self {
        Self {
            success: false,
            stdout: String::new(),
            stderr: stderr.into(),
            exit_code,
            diff_snippet: None,
            ast_valid: true,
            execution_time_ms: 0,
            truncated: false,
        }
    }
}

/// Environment and sandbox context for tool execution.
#[derive(Debug, Clone)]
pub struct ToolExecutionContext {
    pub session_id: String,
    pub workspace_root: PathBuf,
    pub python_binary: PathBuf,
    pub max_output_bytes: usize,
    pub timeout_seconds: u64,
    pub current_tier_approval: SecurityTier,
}

impl Default for ToolExecutionContext {
    fn default() -> Self {
        Self {
            session_id: "default-session".to_string(),
            workspace_root: PathBuf::from("D:/PROJECTS/OM"),
            python_binary: PathBuf::from("C:\\msys64\\ucrt64\\bin\\python.exe"),
            max_output_bytes: 46080,
            timeout_seconds: 30,
            current_tier_approval: SecurityTier::Tier2Guarded,
        }
    }
}

/// Comprehensive tool error taxonomy.
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Permission denied: command requires Tier {required:?} approval but executed with Tier {current:?}: {reason}")]
    PermissionDenied {
        required: SecurityTier,
        current: SecurityTier,
        reason: String,
    },

    #[error("Execution timed out after {timeout_secs}s")]
    Timeout {
        timeout_secs: u64,
        partial_output: String,
    },

    #[error("Target content not found in {file} within lines [{start}..{end}]")]
    TargetNotFound {
        file: String,
        start: usize,
        end: usize,
        snippet: String,
    },

    #[error("Target content matched outside expected range [{expected_start}..{expected_end}] at line {actual_line}")]
    LineRangeMismatch {
        expected_start: usize,
        expected_end: usize,
        actual_line: usize,
    },

    #[error("Ambiguous target match in {file}: found {count} occurrences, allow_multiple is false")]
    AmbiguousMatch {
        file: String,
        count: usize,
        occurrences: Vec<usize>,
    },

    #[error("AST syntax validation failed for {file}: {error_message} at line {line}:{column}")]
    AstValidationError {
        file: String,
        error_message: String,
        line: usize,
        column: usize,
    },

    #[error("File already exists at {path} and overwrite is false")]
    FileAlreadyExists { path: String },

    #[error("File not found at {path}")]
    FileNotFound { path: String },

    #[error("Invalid python invocation: bare 'python' is forbidden. Must use '{expected}'")]
    InvalidPythonInvocation { command: String, expected: String },

    #[error("Process execution failed with exit code {exit_code}: {stderr}")]
    ProcessFailed {
        exit_code: i32,
        stdout: String,
        stderr: String,
    },

    #[error("Path traversal / escape violation: {path} is outside workspace root {root}")]
    PathEscapeViolation { path: String, root: String },

    #[error("Invalid tool arguments: {0}")]
    InvalidArguments(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Standardized async tool interface.
#[async_trait]
pub trait XenoTool: Send + Sync {
    /// Unique identifier for this tool.
    fn name(&self) -> &str;

    /// Definition and JSON Schema parameter specifications.
    fn definition(&self) -> ToolDefinition;

    /// Minimum security tier required to execute this tool.
    fn security_tier(&self) -> SecurityTier;

    /// Executes the tool with the given arguments and execution context.
    async fn execute(&self, args: Value, ctx: &ToolExecutionContext) -> Result<ToolResult, ToolError>;
}
