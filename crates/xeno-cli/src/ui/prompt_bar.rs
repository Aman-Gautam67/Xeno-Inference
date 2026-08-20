//! Interactive prompt bar with slash command auto-completion and status indicator.

/// Prompt bar component.
#[derive(Debug, Clone)]
pub struct PromptBar {
    pub prompt_buffer: String,
    pub status_message: String,
}

impl Default for PromptBar {
    fn default() -> Self {
        Self {
            prompt_buffer: String::new(),
            status_message: "Ready. Type prompt or /swarm, /dag, /diff, /tools, /quit".into(),
        }
    }
}

impl PromptBar {
    /// Renders the prompt input bar string.
    pub fn render(&self) -> String {
        format!(
            "─────────────────────────────────────────────────────────────────────────────────────────────────────────────\n \
             XENO > {}█\n \
             [Status: {}] [Tab: Complete] [Ctrl+C: Interrupt] [Ctrl+Q: Quit]\n",
            self.prompt_buffer, self.status_message
        )
    }

    /// Evaluates slash commands.
    pub fn evaluate_command(&self, input: &str) -> Option<&'static str> {
        let trimmed = input.trim();
        if trimmed == "/quit" || trimmed == "/exit" {
            Some("QUIT")
        } else if trimmed == "/dag" {
            Some("TOGGLE_DAG")
        } else if trimmed == "/diff" {
            Some("TOGGLE_DIFF")
        } else if trimmed.starts_with("/swarm") {
            Some("RUN_SWARM")
        } else if trimmed == "/tools" {
            Some("LIST_TOOLS")
        } else {
            None
        }
    }
}
