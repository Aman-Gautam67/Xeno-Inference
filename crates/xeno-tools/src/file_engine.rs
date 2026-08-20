//! Atomic character-exact file editing engine, atomic file writer, slice reader, and rollback stack.

use crate::ast_validator::AstValidator;
use crate::tool_trait::ToolError;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use uuid::Uuid;

/// Transactional in-memory and disk rollback history stack.
#[derive(Debug, Default)]
pub struct RollbackStack {
    history: Mutex<HashMap<PathBuf, Vec<String>>>,
}

impl RollbackStack {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(HashMap::new()),
        }
    }

    /// Pushes a file snapshot onto the rollback stack.
    pub fn push_snapshot(&self, path: PathBuf, content: String) {
        let mut map = self.history.lock().unwrap();
        map.entry(path).or_default().push(content);
    }

    /// Pops the last snapshot and restores it.
    pub fn rollback_last(&self, path: &Path) -> Result<Option<String>, ToolError> {
        let mut map = self.history.lock().unwrap();
        if let Some(stack) = map.get_mut(path) {
            if let Some(prev) = stack.pop() {
                // Write back to disk
                fs::write(path, &prev)?;
                return Ok(Some(prev));
            }
        }
        Ok(None)
    }
}

/// Character-exact atomic file replacement and modification engine.
#[derive(Debug, Clone, Default)]
pub struct FileEngine {
    ast_validator: AstValidator,
}

impl FileEngine {
    pub fn new() -> Self {
        Self {
            ast_validator: AstValidator::new(),
        }
    }

    /// Performs a character-exact, line-bounded substring replacement in `target_file`.
    #[allow(clippy::too_many_arguments)]
    pub fn multi_replace_file_content(
        &self,
        target_file: &Path,
        target_content: &str,
        replacement_content: &str,
        allow_multiple: bool,
        start_line: Option<usize>,
        end_line: Option<usize>,
        rollback_stack: Option<&RollbackStack>,
    ) -> Result<String, ToolError> {
        if !target_file.exists() {
            return Err(ToolError::FileNotFound {
                path: target_file.to_string_lossy().to_string(),
            });
        }

        let original_bytes = fs::read(target_file)?;
        let original = String::from_utf8(original_bytes).map_err(|e| {
            ToolError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("File is not valid UTF-8: {e}"),
            ))
        })?;

        // Detect line ending convention
        let uses_crlf = original.contains("\r\n");

        // Normalize target and replacement line endings to match document
        let normalized_target = if uses_crlf {
            target_content.replace("\r\n", "\n").replace('\n', "\r\n")
        } else {
            target_content.replace("\r\n", "\n")
        };

        let normalized_replacement = if uses_crlf {
            replacement_content.replace("\r\n", "\n").replace('\n', "\r\n")
        } else {
            replacement_content.replace("\r\n", "\n")
        };

        let file_path_str = target_file.to_string_lossy().to_string();

        // 1. If start_line and end_line are provided, slice document lines
        let (new_content, diff) = if let (Some(s_line), Some(e_line)) = (start_line, end_line) {
            let lines: Vec<&str> = if uses_crlf {
                original.split("\r\n").collect()
            } else {
                original.split('\n').collect()
            };

            let total_lines = lines.len();
            if s_line > total_lines || s_line == 0 || e_line < s_line {
                return Err(ToolError::TargetNotFound {
                    file: file_path_str,
                    start: s_line,
                    end: e_line,
                    snippet: target_content.to_string(),
                });
            }

            // Slice out lines between [s_line - 1, min(e_line, total_lines)]
            let end_idx = e_line.min(total_lines);
            let chunk_delimiter = if uses_crlf { "\r\n" } else { "\n" };
            let chunk = lines[s_line - 1..end_idx].join(chunk_delimiter);

            if !chunk.contains(&normalized_target) {
                // Check if target exists elsewhere in file
                if original.contains(&normalized_target) {
                    let full_matches: Vec<_> = original.match_indices(&normalized_target).collect();
                    let actual_line = original[..full_matches[0].0].lines().count() + 1;
                    return Err(ToolError::LineRangeMismatch {
                        expected_start: s_line,
                        expected_end: e_line,
                        actual_line,
                    });
                }
                return Err(ToolError::TargetNotFound {
                    file: file_path_str,
                    start: s_line,
                    end: e_line,
                    snippet: target_content.to_string(),
                });
            }

            let occurrences = chunk.matches(&normalized_target).count();
            if occurrences > 1 && !allow_multiple {
                let occurrences_lines: Vec<usize> = chunk
                    .match_indices(&normalized_target)
                    .map(|(byte_idx, _)| s_line + chunk[..byte_idx].lines().count())
                    .collect();
                return Err(ToolError::AmbiguousMatch {
                    file: file_path_str,
                    count: occurrences,
                    occurrences: occurrences_lines,
                });
            }

            let replaced_chunk = if allow_multiple {
                chunk.replace(&normalized_target, &normalized_replacement)
            } else {
                chunk.replacen(&normalized_target, &normalized_replacement, 1)
            };

            let mut assembled = Vec::new();
            if s_line > 1 {
                assembled.push(lines[..s_line - 1].join(chunk_delimiter));
                assembled.push(chunk_delimiter.to_string());
            }
            assembled.push(replaced_chunk);
            if end_idx < total_lines {
                assembled.push(chunk_delimiter.to_string());
                assembled.push(lines[end_idx..].join(chunk_delimiter));
            }

            let joined = assembled.join("");
            (joined, generate_simple_diff(&normalized_target, &normalized_replacement))
        } else {
            // Whole file search & replace
            let occurrences = original.matches(&normalized_target).count();
            if occurrences == 0 {
                return Err(ToolError::TargetNotFound {
                    file: file_path_str,
                    start: 1,
                    end: original.lines().count(),
                    snippet: target_content.to_string(),
                });
            }
            if occurrences > 1 && !allow_multiple {
                let occurrences_lines: Vec<usize> = original
                    .match_indices(&normalized_target)
                    .map(|(idx, _)| original[..idx].lines().count() + 1)
                    .collect();
                return Err(ToolError::AmbiguousMatch {
                    file: file_path_str,
                    count: occurrences,
                    occurrences: occurrences_lines,
                });
            }

            let replaced = if allow_multiple {
                original.replace(&normalized_target, &normalized_replacement)
            } else {
                original.replacen(&normalized_target, &normalized_replacement, 1)
            };
            (replaced, generate_simple_diff(&normalized_target, &normalized_replacement))
        };

        // 2. Validate AST syntax of new content
        self.ast_validator.validate_syntax(target_file, &new_content)?;

        // 3. Save snapshot for rollback if stack provided
        if let Some(stack) = rollback_stack {
            stack.push_snapshot(target_file.to_path_buf(), original);
        }

        // 4. Commit atomic write to disk
        self.atomic_write_content(target_file, new_content.as_bytes())?;

        Ok(diff)
    }

    /// Atomically writes content to a file.
    pub fn atomic_write_file(
        &self,
        target_file: &Path,
        content: &str,
        overwrite: bool,
    ) -> Result<(), ToolError> {
        if target_file.exists() && !overwrite {
            return Err(ToolError::FileAlreadyExists {
                path: target_file.to_string_lossy().to_string(),
            });
        }

        // Verify AST before writing
        self.ast_validator.validate_syntax(target_file, content)?;

        if let Some(parent) = target_file.parent() {
            fs::create_dir_all(parent)?;
        }

        self.atomic_write_content(target_file, content.as_bytes())
    }

    /// Reads a slice of a file with line-bounds and truncation budget.
    pub fn file_read_slice(
        &self,
        target_file: &Path,
        start_line: Option<usize>,
        end_line: Option<usize>,
        content_offset: Option<usize>,
        max_bytes: Option<usize>,
    ) -> Result<String, ToolError> {
        if !target_file.exists() {
            return Err(ToolError::FileNotFound {
                path: target_file.to_string_lossy().to_string(),
            });
        }

        let bytes = fs::read(target_file)?;

        // Check for binary content (null bytes in first 8KB)
        let check_len = bytes.len().min(8192);
        if bytes[..check_len].contains(&0) {
            return Ok(format!(
                "[Binary file: {} bytes, mime: application/octet-stream]",
                bytes.len()
            ));
        }

        let content = String::from_utf8_lossy(&bytes).into_owned();
        let max_budget = max_bytes.unwrap_or(46080); // ~10k tokens limit

        // Byte offset slicing if specified
        let sliced_by_offset = if let Some(offset) = content_offset {
            if offset >= content.len() {
                return Ok(String::new());
            }
            &content[offset..]
        } else {
            &content[..]
        };

        // Line-based slicing if specified
        let s_line = start_line.unwrap_or(1);
        let lines: Vec<&str> = sliced_by_offset.lines().collect();
        let total_lines = lines.len();

        if s_line > total_lines {
            return Ok(String::new());
        }

        let e_line = end_line.unwrap_or(total_lines).min(total_lines);
        let selected_lines = &lines[s_line - 1..e_line];
        let mut result = selected_lines.join("\n");

        // Apply max_budget truncation
        if result.len() > max_budget {
            let mut truncate_at = max_budget;
            while !result.is_char_boundary(truncate_at) && truncate_at > 0 {
                truncate_at -= 1;
            }
            result.truncate(truncate_at);
            result.push_str("\n... [Truncated: output exceeded max_bytes limit]");
        }

        Ok(result)
    }

    fn atomic_write_content(&self, target_file: &Path, bytes: &[u8]) -> Result<(), ToolError> {
        let parent = target_file.parent().unwrap_or_else(|| Path::new("."));
        let temp_filename = format!(
            ".tmp_{}_{}",
            Uuid::new_v4(),
            target_file.file_name().and_then(|n| n.to_str()).unwrap_or("file")
        );
        let temp_path = parent.join(temp_filename);

        fs::write(&temp_path, bytes)?;

        // Rename temp file to target path atomically
        if let Err(e) = fs::rename(&temp_path, target_file) {
            // Cleanup temp file on failure
            let _ = fs::remove_file(&temp_path);
            return Err(ToolError::Io(e));
        }

        Ok(())
    }
}

fn generate_simple_diff(old: &str, new: &str) -> String {
    let mut diff = String::new();
    diff.push_str("--- original\n+++ replacement\n");
    for line in old.lines() {
        diff.push_str(&format!("-{line}\n"));
    }
    for line in new.lines() {
        diff.push_str(&format!("+{line}\n"));
    }
    diff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_replace_and_slice() {
        let temp_dir = tempfile_dir();
        let test_file = temp_dir.join("test_code.rs");

        let original_code = "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
        fs::write(&test_file, original_code).unwrap();

        let engine = FileEngine::new();
        let diff = engine.multi_replace_file_content(
            &test_file,
            "    a + b",
            "    a + b + 0",
            false,
            Some(1),
            Some(3),
            None,
        ).unwrap();

        assert!(diff.contains("-    a + b"));
        assert!(diff.contains("+    a + b + 0"));

        let read_back = engine.file_read_slice(&test_file, Some(1), Some(3), None, None).unwrap();
        assert!(read_back.contains("a + b + 0"));

        let _ = fs::remove_file(&test_file);
    }

    fn tempfile_dir() -> PathBuf {
        std::env::temp_dir()
    }
}
