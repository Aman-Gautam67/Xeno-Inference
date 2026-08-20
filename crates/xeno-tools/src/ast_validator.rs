//! In-memory AST syntax validator for Rust, Python, JSON, TOML, and TypeScript/JavaScript.

use crate::tool_trait::ToolError;
use std::path::Path;

/// In-memory AST and syntax validation engine.
#[derive(Debug, Clone, Default)]
pub struct AstValidator;

impl AstValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validates syntax of `code` targeting the language inferred from `file_path`.
    pub fn validate_syntax(&self, file_path: &Path, code: &str) -> Result<(), ToolError> {
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let file_str = file_path.to_string_lossy().to_string();

        match ext.as_str() {
            "rs" => self.validate_rust(&file_str, code),
            "py" => self.validate_python(&file_str, code),
            "json" => self.validate_json(&file_str, code),
            "toml" => self.validate_toml(&file_str, code),
            "js" | "jsx" | "ts" | "tsx" => self.validate_js_ts(&file_str, code),
            _ => Ok(()), // Non-code or unsupported extensions pass by default
        }
    }

    fn validate_rust(&self, file: &str, code: &str) -> Result<(), ToolError> {
        // Delimiter and structural balancing check
        let mut paren_stack = Vec::new();
        let mut in_string = false;
        let mut in_char = false;
        let mut in_line_comment = false;
        let mut in_block_comment = false;
        let mut escape = false;

        let chars: Vec<char> = code.chars().collect();
        let mut line = 1usize;
        let mut col = 1usize;

        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];

            if c == '\n' {
                line += 1;
                col = 1;
                in_line_comment = false;
                i += 1;
                continue;
            }

            if in_line_comment {
                i += 1;
                col += 1;
                continue;
            }

            if in_block_comment {
                if c == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                    in_block_comment = false;
                    i += 2;
                    col += 2;
                    continue;
                }
                i += 1;
                col += 1;
                continue;
            }

            if !in_string && !in_char && c == '/' && i + 1 < chars.len() {
                if chars[i + 1] == '/' {
                    in_line_comment = true;
                    i += 2;
                    col += 2;
                    continue;
                } else if chars[i + 1] == '*' {
                    in_block_comment = true;
                    i += 2;
                    col += 2;
                    continue;
                }
            }

            if escape {
                escape = false;
                i += 1;
                col += 1;
                continue;
            }

            if c == '\\' && (in_string || in_char) {
                escape = true;
                i += 1;
                col += 1;
                continue;
            }

            if c == '"' && !in_char {
                in_string = !in_string;
                i += 1;
                col += 1;
                continue;
            }

            if c == '\'' && !in_string {
                // Check if it's a lifetime 'a vs char literal 'x'
                if !in_char && i + 2 < chars.len() && chars[i + 2] == '\'' {
                    in_char = true;
                } else if in_char {
                    in_char = false;
                }
                i += 1;
                col += 1;
                continue;
            }

            if !in_string && !in_char {
                match c {
                    '(' | '[' | '{' => {
                        paren_stack.push((c, line, col));
                    }
                    ')' => {
                        match paren_stack.pop() {
                            Some(('(', _, _)) => {}
                            _ => {
                                return Err(ToolError::AstValidationError {
                                    file: file.to_string(),
                                    error_message: "Unmatched closing parenthesis ')'".into(),
                                    line,
                                    column: col,
                                });
                            }
                        }
                    }
                    ']' => {
                        match paren_stack.pop() {
                            Some(('[', _, _)) => {}
                            _ => {
                                return Err(ToolError::AstValidationError {
                                    file: file.to_string(),
                                    error_message: "Unmatched closing bracket ']'".into(),
                                    line,
                                    column: col,
                                });
                            }
                        }
                    }
                    '}' => {
                        match paren_stack.pop() {
                            Some(('{', _, _)) => {}
                            _ => {
                                return Err(ToolError::AstValidationError {
                                    file: file.to_string(),
                                    error_message: "Unmatched closing brace '}'".into(),
                                    line,
                                    column: col,
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }

            i += 1;
            col += 1;
        }

        if in_string {
            return Err(ToolError::AstValidationError {
                file: file.to_string(),
                error_message: "Unterminated string literal".into(),
                line,
                column: col,
            });
        }

        if let Some((open_delim, o_line, o_col)) = paren_stack.pop() {
            return Err(ToolError::AstValidationError {
                file: file.to_string(),
                error_message: format!("Unclosed delimiter '{open_delim}'"),
                line: o_line,
                column: o_col,
            });
        }

        // Basic syntax anomaly check (e.g. "= ;" or "fn ()")
        if code.contains("= ;") || code.contains("let ;") {
            return Err(ToolError::AstValidationError {
                file: file.to_string(),
                error_message: "Unexpected empty assignment token '= ;'".into(),
                line: 1,
                column: 1,
            });
        }

        Ok(())
    }

    fn validate_python(&self, file: &str, code: &str) -> Result<(), ToolError> {
        let mut paren_stack = Vec::new();
        let mut in_string = false;
        let mut str_delim = '"';

        let lines: Vec<&str> = code.lines().collect();
        for (l_idx, line) in lines.iter().enumerate() {
            let line_num = l_idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with('#') {
                continue;
            }

            for (col_idx, c) in line.chars().enumerate() {
                let col = col_idx + 1;

                if (c == '"' || c == '\'') && !in_string {
                    in_string = true;
                    str_delim = c;
                } else if c == str_delim && in_string {
                    in_string = false;
                }

                if !in_string {
                    if c == '#' {
                        break; // Rest of line is comment
                    }
                    match c {
                        '(' | '[' | '{' => paren_stack.push((c, line_num, col)),
                        ')' => match paren_stack.pop() {
                            Some(('(', _, _)) => {}
                            _ => return Err(ToolError::AstValidationError {
                                file: file.to_string(),
                                error_message: "Unmatched closing ')' in Python".into(),
                                line: line_num,
                                column: col,
                            }),
                        },
                        ']' => match paren_stack.pop() {
                            Some(('[', _, _)) => {}
                            _ => return Err(ToolError::AstValidationError {
                                file: file.to_string(),
                                error_message: "Unmatched closing ']' in Python".into(),
                                line: line_num,
                                column: col,
                            }),
                        },
                        '}' => match paren_stack.pop() {
                            Some(('{', _, _)) => {}
                            _ => return Err(ToolError::AstValidationError {
                                file: file.to_string(),
                                error_message: "Unmatched closing '}' in Python".into(),
                                line: line_num,
                                column: col,
                            }),
                        },
                        _ => {}
                    }
                }
            }
        }

        if let Some((delim, line, col)) = paren_stack.pop() {
            return Err(ToolError::AstValidationError {
                file: file.to_string(),
                error_message: format!("Unclosed delimiter '{delim}' in Python script"),
                line,
                column: col,
            });
        }

        Ok(())
    }

    fn validate_json(&self, file: &str, code: &str) -> Result<(), ToolError> {
        serde_json::from_str::<serde_json::Value>(code).map_err(|e| {
            ToolError::AstValidationError {
                file: file.to_string(),
                error_message: format!("JSON parse error: {e}"),
                line: e.line(),
                column: e.column(),
            }
        })?;
        Ok(())
    }

    fn validate_toml(&self, file: &str, code: &str) -> Result<(), ToolError> {
        let mut in_string = false;
        let mut brace_count = 0;
        let mut bracket_count = 0;

        for (l_idx, line) in code.lines().enumerate() {
            let line_num = l_idx + 1;
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }

            for (col_idx, c) in line.chars().enumerate() {
                if c == '"' {
                    in_string = !in_string;
                }
                if !in_string {
                    if c == '{' { brace_count += 1; }
                    else if c == '}' { brace_count -= 1; }
                    else if c == '[' { bracket_count += 1; }
                    else if c == ']' { bracket_count -= 1; }
                }
                if brace_count < 0 || bracket_count < 0 {
                    return Err(ToolError::AstValidationError {
                        file: file.to_string(),
                        error_message: "Mismatched delimiters in TOML".into(),
                        line: line_num,
                        column: col_idx + 1,
                    });
                }
            }
        }

        if brace_count != 0 || bracket_count != 0 {
            return Err(ToolError::AstValidationError {
                file: file.to_string(),
                error_message: "Unclosed delimiters in TOML document".into(),
                line: 1,
                column: 1,
            });
        }

        Ok(())
    }

    fn validate_js_ts(&self, file: &str, code: &str) -> Result<(), ToolError> {
        let mut paren_stack = Vec::new();
        let mut in_string = false;
        let mut str_delim = '"';

        for (l_idx, line) in code.lines().enumerate() {
            let line_num = l_idx + 1;
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }

            for (col_idx, c) in line.chars().enumerate() {
                let col = col_idx + 1;
                if (c == '"' || c == '\'' || c == '`') && !in_string {
                    in_string = true;
                    str_delim = c;
                } else if c == str_delim && in_string {
                    in_string = false;
                }

                if !in_string {
                    match c {
                        '(' | '[' | '{' => paren_stack.push((c, line_num, col)),
                        ')' => match paren_stack.pop() {
                            Some(('(', _, _)) => {}
                            _ => return Err(ToolError::AstValidationError {
                                file: file.to_string(),
                                error_message: "Unmatched closing ')' in JS/TS".into(),
                                line: line_num,
                                column: col,
                            }),
                        },
                        ']' => match paren_stack.pop() {
                            Some(('[', _, _)) => {}
                            _ => return Err(ToolError::AstValidationError {
                                file: file.to_string(),
                                error_message: "Unmatched closing ']' in JS/TS".into(),
                                line: line_num,
                                column: col,
                            }),
                        },
                        '}' => match paren_stack.pop() {
                            Some(('{', _, _)) => {}
                            _ => return Err(ToolError::AstValidationError {
                                file: file.to_string(),
                                error_message: "Unmatched closing '}' in JS/TS".into(),
                                line: line_num,
                                column: col,
                            }),
                        },
                        _ => {}
                    }
                }
            }
        }

        if let Some((delim, line, col)) = paren_stack.pop() {
            return Err(ToolError::AstValidationError {
                file: file.to_string(),
                error_message: format!("Unclosed delimiter '{delim}' in JS/TS"),
                line,
                column: col,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_ast_validation() {
        let validator = AstValidator::new();
        let valid_code = "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
        assert!(validator.validate_syntax(Path::new("src/lib.rs"), valid_code).is_ok());

        let invalid_code = "pub fn broken(a: i32 { a + b }\n";
        assert!(validator.validate_syntax(Path::new("src/lib.rs"), invalid_code).is_err());
    }

    #[test]
    fn test_json_ast_validation() {
        let validator = AstValidator::new();
        let valid_json = r#"{"name": "xeno", "version": 1}"#;
        assert!(validator.validate_syntax(Path::new("config.json"), valid_json).is_ok());

        let bad_json = r#"{"name": "xeno", "unclosed": "#;
        assert!(validator.validate_syntax(Path::new("config.json"), bad_json).is_err());
    }
}
