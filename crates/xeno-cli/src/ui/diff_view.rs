//! Active AST diff and file change viewer.

/// Diff viewer component for terminal rendering.
#[derive(Debug, Clone, Default)]
pub struct DiffView {
    pub file_path: String,
    pub diff_snippet: String,
}

impl DiffView {
    pub fn new(file_path: impl Into<String>, diff: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            diff_snippet: diff.into(),
        }
    }

    /// Formats the diff snippet for terminal display.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "┌─ ACTIVE AST DIFF: {} ────────────────────────\n",
            self.file_path
        ));

        if self.diff_snippet.is_empty() {
            out.push_str("  (No active diff)\n");
        } else {
            for line in self.diff_snippet.lines() {
                if line.starts_with('+') {
                    out.push_str(&format!("  \x1b[32m{line}\x1b[0m\n")); // Green
                } else if line.starts_with('-') {
                    out.push_str(&format!("  \x1b[31m{line}\x1b[0m\n")); // Red
                } else {
                    out.push_str(&format!("  {line}\n"));
                }
            }
        }

        out.push_str("└────────────────────────────────────────────────────────┘\n");
        out
    }
}
