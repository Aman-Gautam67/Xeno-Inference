//! Search and code intelligence engine (`fuzzy_glob_ripgrep`).

use crate::tool_trait::ToolError;
use regex::{Regex, RegexBuilder};
use std::fs;
use std::path::Path;

/// Individual matching item from a search.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchMatch {
    pub file_path: String,
    pub line_number: Option<usize>,
    pub line_content: Option<String>,
}

/// Directory search and ripgrep engine.
#[derive(Debug, Clone, Default)]
pub struct SearchEngine;

impl SearchEngine {
    pub fn new() -> Self {
        Self
    }

    /// Searches `search_path` for files matching `pattern` and containing `query`.
    #[allow(clippy::too_many_arguments)]
    pub fn search(
        &self,
        search_path: &Path,
        pattern: Option<&str>,
        query: &str,
        is_regex: bool,
        case_insensitive: bool,
        match_per_line: bool,
        max_matches: usize,
    ) -> Result<Vec<SearchMatch>, ToolError> {
        if !search_path.exists() {
            return Err(ToolError::FileNotFound {
                path: search_path.to_string_lossy().to_string(),
            });
        }

        // Build search regex
        let re = if is_regex {
            RegexBuilder::new(query)
                .case_insensitive(case_insensitive)
                .build()
                .map_err(|e| ToolError::InvalidArguments(format!("Invalid regex query: {e}")))?
        } else {
            let escaped = regex::escape(query);
            RegexBuilder::new(&escaped)
                .case_insensitive(case_insensitive)
                .build()
                .map_err(|e| ToolError::InvalidArguments(format!("Regex build error: {e}")))?
        };

        let mut matches = Vec::new();
        self.walk_and_search(
            search_path,
            pattern,
            &re,
            match_per_line,
            max_matches,
            &mut matches,
        )?;

        Ok(matches)
    }

    fn walk_and_search(
        &self,
        dir: &Path,
        pattern: Option<&str>,
        re: &Regex,
        match_per_line: bool,
        max_matches: usize,
        matches: &mut Vec<SearchMatch>,
    ) -> Result<(), ToolError> {
        if matches.len() >= max_matches {
            return Ok(());
        }

        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(()), // Skip inaccessible dirs
        };

        for entry in entries.flatten() {
            if matches.len() >= max_matches {
                break;
            }

            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Ignore hidden and git/target folders
            if file_name.starts_with('.') || file_name == "target" || file_name == "node_modules" {
                continue;
            }

            if path.is_dir() {
                self.walk_and_search(&path, pattern, re, match_per_line, max_matches, matches)?;
            } else if path.is_file() {
                // Check glob pattern if specified
                if let Some(pat) = pattern {
                    if !matches_glob(&file_name, pat) && !matches_glob(&path.to_string_lossy(), pat) {
                        continue;
                    }
                }

                // Check file content
                if let Ok(bytes) = fs::read(&path) {
                    // Skip binary files
                    let check_len = bytes.len().min(1024);
                    if bytes[..check_len].contains(&0) {
                        continue;
                    }

                    if let Ok(content) = std::str::from_utf8(&bytes) {
                        let path_str = path.to_string_lossy().replace('\\', "/");

                        if match_per_line {
                            for (idx, line) in content.lines().enumerate() {
                                if re.is_match(line) {
                                    matches.push(SearchMatch {
                                        file_path: path_str.clone(),
                                        line_number: Some(idx + 1),
                                        line_content: Some(line.to_string()),
                                    });

                                    if matches.len() >= max_matches {
                                        break;
                                    }
                                }
                            }
                        } else if re.is_match(content) {
                            matches.push(SearchMatch {
                                file_path: path_str,
                                line_number: None,
                                line_content: None,
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn matches_glob(path: &str, pattern: &str) -> bool {
    if pattern == "*" || pattern == "**/*" {
        return true;
    }

    if let Some(ext) = pattern.strip_prefix("*.") {
        return path.ends_with(&format!(".{ext}"));
    }

    if let Some(ext) = pattern.strip_prefix("**/*.") {
        return path.ends_with(&format!(".{ext}"));
    }

    path.contains(pattern.trim_matches('*'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_engine() {
        let searcher = SearchEngine::new();
        let matches = searcher
            .search(
                Path::new("."),
                Some("*.toml"),
                "workspace",
                false,
                false,
                true,
                10,
            )
            .unwrap();

        assert!(!matches.is_empty());
        assert!(matches[0].file_path.contains("Cargo.toml"));
    }
}
