//! XENO INFERENCE — Sandboxed PTY & Atomic AST File Engine (`xeno-tools`).
//!
//! Provides isolated virtual PTY sessions, character-exact atomic AST file manipulation,
//! fuzzy glob / ripgrep search, Python execution sandboxing, and MCP tool runtime.

pub mod ast_validator;
pub mod file_engine;
pub mod mcp;
pub mod pty;
pub mod python_runner;
pub mod safety;
pub mod search;
pub mod tool_trait;

use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;

pub use ast_validator::AstValidator;
pub use file_engine::{FileEngine, RollbackStack};
pub use mcp::{McpToolRegistry, McpToolSchema};
pub use pty::{PtyManager, PtySession};
pub use python_runner::PythonRunner;
pub use safety::{PathValidator, PythonSanitizer, SecurityClassifier, REQUIRED_PYTHON_PATH};
pub use search::{SearchEngine, SearchMatch};
pub use tool_trait::{
    SecurityTier, ToolDefinition, ToolError, ToolExecutionContext, ToolResult, XenoTool,
};

/// Prelude exporting all standard tool engine primitives.
pub mod prelude {
    pub use super::ast_validator::AstValidator;
    pub use super::file_engine::{FileEngine, RollbackStack};
    pub use super::mcp::{McpToolRegistry, McpToolSchema};
    pub use super::pty::{PtyManager, PtySession};
    pub use super::python_runner::PythonRunner;
    pub use super::safety::{PathValidator, PythonSanitizer, SecurityClassifier, REQUIRED_PYTHON_PATH};
    pub use super::search::{SearchEngine, SearchMatch};
    pub use super::tool_trait::{
        SecurityTier, ToolDefinition, ToolError, ToolExecutionContext, ToolResult, XenoTool,
    };
    pub use super::{
        AtomicWriteTool, FileReadSliceTool, FuzzyGlobRipgrepTool, MultiReplaceTool,
        PythonRunnerTool, TerminalExecTool,
    };
}

// =========================================================================
// BUILT-IN TOOL IMPLEMENTATIONS
// =========================================================================

/// Tool implementation for `multi_replace_file_content`.
#[derive(Debug, Clone, Default)]
pub struct MultiReplaceTool {
    engine: FileEngine,
}

impl MultiReplaceTool {
    pub fn new() -> Self {
        Self {
            engine: FileEngine::new(),
        }
    }
}

#[async_trait]
impl XenoTool for MultiReplaceTool {
    fn name(&self) -> &str {
        "multi_replace_file_content"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "multi_replace_file_content".to_string(),
            description: "Character-exact substring replacement with in-memory AST syntax validation".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "TargetFile": { "type": "string", "description": "Absolute path to target file" },
                    "TargetContent": { "type": "string", "description": "Exact character sequence to replace" },
                    "ReplacementContent": { "type": "string", "description": "Replacement content" },
                    "AllowMultiple": { "type": "boolean", "default": false },
                    "StartLine": { "type": "integer", "description": "1-indexed starting line" },
                    "EndLine": { "type": "integer", "description": "1-indexed ending line" }
                },
                "required": ["TargetFile", "TargetContent", "ReplacementContent"]
            }),
            tier: SecurityTier::Tier2Guarded,
        }
    }

    fn security_tier(&self) -> SecurityTier {
        SecurityTier::Tier2Guarded
    }

    async fn execute(&self, args: Value, _ctx: &ToolExecutionContext) -> Result<ToolResult, ToolError> {
        let target_file_str = args["TargetFile"].as_str().ok_or_else(|| {
            ToolError::InvalidArguments("Missing 'TargetFile' parameter".into())
        })?;
        let target_content = args["TargetContent"].as_str().ok_or_else(|| {
            ToolError::InvalidArguments("Missing 'TargetContent' parameter".into())
        })?;
        let replacement_content = args["ReplacementContent"].as_str().ok_or_else(|| {
            ToolError::InvalidArguments("Missing 'ReplacementContent' parameter".into())
        })?;
        let allow_multiple = args["AllowMultiple"].as_bool().unwrap_or(false);
        let start_line = args["StartLine"].as_u64().map(|v| v as usize);
        let end_line = args["EndLine"].as_u64().map(|v| v as usize);

        let target_path = PathBuf::from(target_file_str);
        let diff = self.engine.multi_replace_file_content(
            &target_path,
            target_content,
            replacement_content,
            allow_multiple,
            start_line,
            end_line,
            None,
        )?;

        let mut res = ToolResult::success(format!("File modified successfully: {target_file_str}"));
        res.diff_snippet = Some(diff);
        Ok(res)
    }
}

/// Tool implementation for `atomic_write_file`.
#[derive(Debug, Clone, Default)]
pub struct AtomicWriteTool {
    engine: FileEngine,
}

impl AtomicWriteTool {
    pub fn new() -> Self {
        Self {
            engine: FileEngine::new(),
        }
    }
}

#[async_trait]
impl XenoTool for AtomicWriteTool {
    fn name(&self) -> &str {
        "atomic_write_file"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "atomic_write_file".to_string(),
            description: "Atomically writes content to disk via temporary swap".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "TargetFile": { "type": "string" },
                    "Content": { "type": "string" },
                    "Overwrite": { "type": "boolean", "default": false }
                },
                "required": ["TargetFile", "Content"]
            }),
            tier: SecurityTier::Tier2Guarded,
        }
    }

    fn security_tier(&self) -> SecurityTier {
        SecurityTier::Tier2Guarded
    }

    async fn execute(&self, args: Value, _ctx: &ToolExecutionContext) -> Result<ToolResult, ToolError> {
        let target_file_str = args["TargetFile"].as_str().ok_or_else(|| {
            ToolError::InvalidArguments("Missing 'TargetFile' parameter".into())
        })?;
        let content = args["Content"].as_str().ok_or_else(|| {
            ToolError::InvalidArguments("Missing 'Content' parameter".into())
        })?;
        let overwrite = args["Overwrite"].as_bool().unwrap_or(false);

        let target_path = PathBuf::from(target_file_str);
        self.engine.atomic_write_file(&target_path, content, overwrite)?;

        Ok(ToolResult::success(format!("File written atomically: {target_file_str}")))
    }
}

/// Tool implementation for `file_read_slice`.
#[derive(Debug, Clone, Default)]
pub struct FileReadSliceTool {
    engine: FileEngine,
}

impl FileReadSliceTool {
    pub fn new() -> Self {
        Self {
            engine: FileEngine::new(),
        }
    }
}

#[async_trait]
impl XenoTool for FileReadSliceTool {
    fn name(&self) -> &str {
        "file_read_slice"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_read_slice".to_string(),
            description: "Reads line-bounded or byte-offset slice from a file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "AbsolutePath": { "type": "string" },
                    "StartLine": { "type": "integer" },
                    "EndLine": { "type": "integer" },
                    "ContentOffset": { "type": "integer" },
                    "MaxBytes": { "type": "integer" }
                },
                "required": ["AbsolutePath"]
            }),
            tier: SecurityTier::Tier1Safe,
        }
    }

    fn security_tier(&self) -> SecurityTier {
        SecurityTier::Tier1Safe
    }

    async fn execute(&self, args: Value, ctx: &ToolExecutionContext) -> Result<ToolResult, ToolError> {
        let path_str = args["AbsolutePath"].as_str().ok_or_else(|| {
            ToolError::InvalidArguments("Missing 'AbsolutePath' parameter".into())
        })?;
        let start_line = args["StartLine"].as_u64().map(|v| v as usize);
        let end_line = args["EndLine"].as_u64().map(|v| v as usize);
        let content_offset = args["ContentOffset"].as_u64().map(|v| v as usize);
        let max_bytes = args["MaxBytes"].as_u64().map(|v| v as usize).unwrap_or(ctx.max_output_bytes);

        let target_path = PathBuf::from(path_str);
        let content = self.engine.file_read_slice(
            &target_path,
            start_line,
            end_line,
            content_offset,
            Some(max_bytes),
        )?;

        let is_truncated = content.contains("... [Truncated");
        let mut res = ToolResult::success(content);
        res.truncated = is_truncated;
        Ok(res)
    }
}

/// Tool implementation for `fuzzy_glob_ripgrep`.
#[derive(Debug, Clone, Default)]
pub struct FuzzyGlobRipgrepTool {
    engine: SearchEngine,
}

impl FuzzyGlobRipgrepTool {
    pub fn new() -> Self {
        Self {
            engine: SearchEngine::new(),
        }
    }
}

#[async_trait]
impl XenoTool for FuzzyGlobRipgrepTool {
    fn name(&self) -> &str {
        "fuzzy_glob_ripgrep"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "fuzzy_glob_ripgrep".to_string(),
            description: "High-speed directory search with glob pattern filtering and regex search".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "SearchPath": { "type": "string" },
                    "Pattern": { "type": "string" },
                    "Query": { "type": "string" },
                    "IsRegex": { "type": "boolean", "default": false },
                    "CaseInsensitive": { "type": "boolean", "default": false },
                    "MatchPerLine": { "type": "boolean", "default": true },
                    "MaxMatches": { "type": "integer", "default": 50 }
                },
                "required": ["SearchPath", "Query"]
            }),
            tier: SecurityTier::Tier1Safe,
        }
    }

    fn security_tier(&self) -> SecurityTier {
        SecurityTier::Tier1Safe
    }

    async fn execute(&self, args: Value, _ctx: &ToolExecutionContext) -> Result<ToolResult, ToolError> {
        let search_path_str = args["SearchPath"].as_str().ok_or_else(|| {
            ToolError::InvalidArguments("Missing 'SearchPath' parameter".into())
        })?;
        let pattern = args["Pattern"].as_str();
        let query = args["Query"].as_str().ok_or_else(|| {
            ToolError::InvalidArguments("Missing 'Query' parameter".into())
        })?;
        let is_regex = args["IsRegex"].as_bool().unwrap_or(false);
        let case_insensitive = args["CaseInsensitive"].as_bool().unwrap_or(false);
        let match_per_line = args["MatchPerLine"].as_bool().unwrap_or(true);
        let max_matches = args["MaxMatches"].as_u64().map(|v| v as usize).unwrap_or(50);

        let matches = self.engine.search(
            PathBuf::from(search_path_str).as_path(),
            pattern,
            query,
            is_regex,
            case_insensitive,
            match_per_line,
            max_matches,
        )?;

        let serialized = serde_json::to_string_pretty(&matches).unwrap_or_default();
        Ok(ToolResult::success(serialized))
    }
}

/// Tool implementation for `terminal_exec`.
#[derive(Debug, Clone, Default)]
pub struct TerminalExecTool {
    pty: Arc<PtyManager>,
}

impl TerminalExecTool {
    pub fn new() -> Self {
        Self {
            pty: Arc::new(PtyManager::new()),
        }
    }
}

#[async_trait]
impl XenoTool for TerminalExecTool {
    fn name(&self) -> &str {
        "terminal_exec"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "terminal_exec".to_string(),
            description: "Executes shell commands in virtual PTY with security checks".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "CommandLine": { "type": "string" },
                    "Cwd": { "type": "string" },
                    "TimeoutSeconds": { "type": "integer", "default": 30 }
                },
                "required": ["CommandLine"]
            }),
            tier: SecurityTier::Tier2Guarded,
        }
    }

    fn security_tier(&self) -> SecurityTier {
        SecurityTier::Tier2Guarded
    }

    async fn execute(&self, args: Value, ctx: &ToolExecutionContext) -> Result<ToolResult, ToolError> {
        let command_line = args["CommandLine"].as_str().ok_or_else(|| {
            ToolError::InvalidArguments("Missing 'CommandLine' parameter".into())
        })?;
        let cwd = args["Cwd"].as_str().map(PathBuf::from);
        let timeout_secs = args["TimeoutSeconds"].as_u64().unwrap_or(ctx.timeout_seconds);

        self.pty
            .execute_command(
                command_line,
                cwd.as_deref(),
                Some(timeout_secs),
                ctx.current_tier_approval,
            )
            .await
    }
}

/// Tool implementation for `python_runner`.
#[derive(Debug, Clone, Default)]
pub struct PythonRunnerTool {
    runner: PythonRunner,
}

impl PythonRunnerTool {
    pub fn new() -> Self {
        Self {
            runner: PythonRunner::default(),
        }
    }
}

#[async_trait]
impl XenoTool for PythonRunnerTool {
    fn name(&self) -> &str {
        "python_runner"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "python_runner".to_string(),
            description: "Executes Python scripts using strict sandbox executable".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "ScriptPath": { "type": "string" },
                    "InlineCode": { "type": "string" },
                    "Args": { "type": "array", "items": { "type": "string" } }
                }
            }),
            tier: SecurityTier::Tier2Guarded,
        }
    }

    fn security_tier(&self) -> SecurityTier {
        SecurityTier::Tier2Guarded
    }

    async fn execute(&self, args: Value, ctx: &ToolExecutionContext) -> Result<ToolResult, ToolError> {
        if let Some(inline) = args["InlineCode"].as_str() {
            self.runner.run_inline_code(inline, Some(ctx.timeout_seconds)).await
        } else if let Some(script) = args["ScriptPath"].as_str() {
            let script_args: Vec<String> = args["Args"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            self.runner
                .run_script(
                    PathBuf::from(script).as_path(),
                    &script_args,
                    None,
                    Some(ctx.timeout_seconds),
                )
                .await
        } else {
            Err(ToolError::InvalidArguments(
                "Either 'InlineCode' or 'ScriptPath' must be provided".into(),
            ))
        }
    }
}
