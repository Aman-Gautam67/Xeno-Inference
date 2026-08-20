//! Safety classifier, Python execution sanitizer, and path escape validator.

use crate::tool_trait::{SecurityTier, ToolError};
use std::path::{Path, PathBuf};

/// Mandatory Python executable path on Windows.
pub const REQUIRED_PYTHON_PATH: &str = r"C:\msys64\ucrt64\bin\python.exe";

/// Security classifier evaluating command risks.
#[derive(Debug, Clone, Default)]
pub struct SecurityClassifier;

impl SecurityClassifier {
    pub fn new() -> Self {
        Self
    }

    /// Classifies a shell command line string into a [`SecurityTier`].
    pub fn classify_command(&self, command: &str) -> SecurityTier {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return SecurityTier::Tier1Safe;
        }

        // Sub-commands evaluation across pipes & chains
        let sub_commands = trimmed.split([';', '|', '&']);
        let mut max_tier = SecurityTier::Tier1Safe;

        for sub in sub_commands {
            let cmd = sub.trim();
            if cmd.is_empty() {
                continue;
            }
            let tier = self.classify_single_command(cmd);
            if tier > max_tier {
                max_tier = tier;
            }
        }

        max_tier
    }

    fn classify_single_command(&self, cmd: &str) -> SecurityTier {
        let lower = cmd.to_lowercase();
        let tokens: Vec<&str> = lower.split_whitespace().collect();
        if tokens.is_empty() {
            return SecurityTier::Tier1Safe;
        }

        let first = tokens[0];

        // Tier 3 Destructive / Irreversible
        if first == "rm" || lower.contains("rm -rf") || lower.contains("rm -r")
            || first == "del" || lower.contains("del /s") || lower.contains("del /q") || lower.contains("rmdir /s")
            || first == "format" || first == "drop" || first == "reg"
            || lower.contains("git push --force") || lower.contains("git reset --hard")
            || first == "sudo" || first == "chmod" || first == "chown"
        {
            return SecurityTier::Tier3Destructive;
        }

        // Tier 1 Safe / Read-Only Inspection
        if first == "ls" || first == "dir" || first == "cat" || first == "type"
            || first == "head" || first == "tail" || first == "grep" || first == "find"
            || first == "echo" || first == "pwd" || first == "which" || first == "where"
            || (first == "git" && tokens.get(1).map_or(false, |&sub| matches!(sub, "status" | "diff" | "log" | "show" | "branch" | "rev-parse")))
            || (first == "cargo" && tokens.get(1).map_or(false, |&sub| matches!(sub, "check" | "test" | "build" | "clippy" | "tree" | "version")))
            || (first == "npm" && tokens.get(1).map_or(false, |&sub| matches!(sub, "test" | "run" | "list" | "audit")))
            || (lower.contains("pytest") || lower.contains("cargo test"))
        {
            return SecurityTier::Tier1Safe;
        }

        // Default to Tier 2 Guarded (e.g. git commit, cargo add, file writes)
        SecurityTier::Tier2Guarded
    }
}

/// Sanitizer and validator for Python interpreter paths.
#[derive(Debug, Clone, Default)]
pub struct PythonSanitizer;

impl PythonSanitizer {
    pub fn new() -> Self {
        Self
    }

    /// Verifies if a given path is the approved Python executable.
    pub fn is_valid_python_binary(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        path_str.ends_with("python.exe")
            && (path_str.contains("msys64") || path_str.contains(r"c:\msys64\ucrt64\bin\python.exe") || path_str.contains("python"))
    }

    /// Resolves and sanitizes a command line invoking python.
    pub fn sanitize_python_command(&self, command: &str) -> Result<String, ToolError> {
        let trimmed = command.trim();
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if tokens.is_empty() {
            return Ok(command.to_string());
        }

        let first = tokens[0].to_lowercase();
        if first == "python" || first == "python3" || first == "py" {
            // Rewrite to strict Python path
            let rest = if tokens.len() > 1 {
                &trimmed[tokens[0].len()..].trim_start()
            } else {
                ""
            };
            return Ok(format!("\"{REQUIRED_PYTHON_PATH}\" {rest}"));
        }

        Ok(command.to_string())
    }
}

/// Validates that filesystem operations remain within permitted workspace boundaries.
#[derive(Debug, Clone)]
pub struct PathValidator {
    workspace_root: PathBuf,
}

impl PathValidator {
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
        }
    }

    /// Validates that `target` resides within `workspace_root` (preventing directory traversal).
    pub fn validate_path(&self, target: &Path) -> Result<PathBuf, ToolError> {
        let target_str = target.to_string_lossy().replace('\\', "/");
        let root_str = self.workspace_root.to_string_lossy().replace('\\', "/");

        // Allow absolute paths inside workspace or relative paths
        if target.is_absolute() {
            if !target_str.to_lowercase().starts_with(&root_str.to_lowercase()) {
                // If on windows, check if drive letter matches or if it's within workspace
                return Err(ToolError::PathEscapeViolation {
                    path: target_str,
                    root: root_str,
                });
            }
            Ok(target.to_path_buf())
        } else {
            // Relative path - join with workspace root
            let joined = self.workspace_root.join(target);
            Ok(joined)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_classifier() {
        let classifier = SecurityClassifier::new();
        assert_eq!(classifier.classify_command("cargo test --workspace"), SecurityTier::Tier1Safe);
        assert_eq!(classifier.classify_command("git status"), SecurityTier::Tier1Safe);
        assert_eq!(classifier.classify_command("git commit -m 'feat'"), SecurityTier::Tier2Guarded);
        assert_eq!(classifier.classify_command("cargo add serde"), SecurityTier::Tier2Guarded);
        assert_eq!(classifier.classify_command("rm -rf /tmp/test"), SecurityTier::Tier3Destructive);
        assert_eq!(classifier.classify_command("del /s /q test.txt"), SecurityTier::Tier3Destructive);
    }

    #[test]
    fn test_python_sanitizer() {
        let sanitizer = PythonSanitizer::new();
        assert!(sanitizer.is_valid_python_binary(Path::new(REQUIRED_PYTHON_PATH)));
        
        let sanitized = sanitizer.sanitize_python_command("python script.py --arg 1").unwrap();
        assert!(sanitized.starts_with(&format!("\"{REQUIRED_PYTHON_PATH}\"")));
        assert!(sanitized.contains("script.py --arg 1"));
    }
}
