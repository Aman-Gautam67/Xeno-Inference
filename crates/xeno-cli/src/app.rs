//! CLI Application state, active view tabs, and terminal event model.

use crate::ui::{render_header, DagView, DiffView, HudState, PromptBar};
use std::sync::Arc;
use xeno_agent::prelude::*;
use xeno_dag::prelude::*;
use xeno_telemetry::prelude::*;

/// Active display pane tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePane {
    Timeline,
    Dag,
    Diff,
    Prompt,
}

impl Default for ActivePane {
    fn default() -> Self {
        ActivePane::Prompt
    }
}

/// Main application state for `xeno-cli`.
pub struct XenoCliApp {
    pub active_pane: ActivePane,
    pub hud: HudState,
    pub dag_view: DagView,
    pub diff_view: DiffView,
    pub prompt_bar: PromptBar,
    pub dag: XenoDAGGraph,
    pub telemetry: Arc<TelemetryCollector>,
    pub timeline_events: Vec<String>,
    pub should_exit: bool,
}

impl Default for XenoCliApp {
    fn default() -> Self {
        Self::new()
    }
}

impl XenoCliApp {
    pub fn new() -> Self {
        let telemetry = Arc::new(TelemetryCollector::new(500));
        Self {
            active_pane: ActivePane::Prompt,
            hud: HudState {
                active_provider: "Mock/Local GGUF".into(),
                velocity: 125.0,
                ttft_ms: 42,
                estimated_cost: 0.0012,
                hardware: Default::default(),
            },
            dag_view: DagView::new(),
            diff_view: DiffView::default(),
            prompt_bar: PromptBar::default(),
            dag: XenoDAGGraph::new(),
            telemetry,
            timeline_events: vec!["[System] XENO INFERENCE CLI initialized".into()],
            should_exit: false,
        }
    }

    /// Renders the complete terminal frame as a string.
    pub fn render_frame(&self) -> String {
        let mut frame = String::new();
        frame.push_str(&render_header("SWARM", &self.hud.active_provider, self.hud.velocity, self.hud.estimated_cost));
        frame.push_str(&self.hud.render());
        frame.push_str(&self.dag_view.render_graph(&self.dag));
        frame.push_str(&self.diff_view.render());
        frame.push_str(&self.prompt_bar.render());
        frame
    }

    /// Processes user text input from prompt bar.
    pub async fn handle_input(&mut self, input: &str) -> String {
        if let Some(cmd) = self.prompt_bar.evaluate_command(input) {
            match cmd {
                "QUIT" => {
                    self.should_exit = true;
                    "Exiting XENO CLI...".into()
                }
                "TOGGLE_DAG" => {
                    self.active_pane = ActivePane::Dag;
                    "Switched to DAG view".into()
                }
                "TOGGLE_DIFF" => {
                    self.active_pane = ActivePane::Diff;
                    "Switched to Diff view".into()
                }
                "RUN_SWARM" => {
                    let goal = input.trim_start_matches("/swarm").trim();
                    let mut harness = XenoAgentHarness::new("cli-session", goal);
                    match harness.execute_goal(goal).await {
                        Ok(res) => {
                            self.hud.estimated_cost += res.telemetry_summary.total_cost_usd;
                            format!("Swarm execution succeeded: {}", res.final_output)
                        }
                        Err(e) => format!("Swarm execution failed: {e}"),
                    }
                }
                _ => "Command processed".into(),
            }
        } else {
            // Standard user prompt
            let mut harness = XenoAgentHarness::new("cli-session", input);
            match harness.execute_goal(input).await {
                Ok(res) => format!("Result: {}", res.final_output),
                Err(e) => format!("Error: {e}"),
            }
        }
    }
}
