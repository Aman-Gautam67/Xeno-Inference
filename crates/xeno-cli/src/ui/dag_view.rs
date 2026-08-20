//! Unicode / ASCII DAG graph renderer.

use xeno_dag::prelude::*;

/// DAG view component for terminal rendering.
#[derive(Debug, Clone, Default)]
pub struct DagView;

impl DagView {
    pub fn new() -> Self {
        Self
    }

    /// Formats nodes in the DAG as Unicode execution graph.
    pub fn render_graph(&self, graph: &XenoDAGGraph) -> String {
        let mut out = String::new();
        out.push_str("┌─ LIVE EXECUTION DAG ───────────────────────────────────┐\n");

        if let Ok(order) = graph.topological_sort() {
            for (idx, node_id) in order.iter().enumerate() {
                if let Some(node) = graph.get_node(node_id) {
                    let status_badge = match node.status {
                        NodeStatus::Pending => "○ PENDING",
                        NodeStatus::Running => "● RUNNING",
                        NodeStatus::Success => "✔ SUCCESS",
                        NodeStatus::Failed => "✖ FAILED",
                        NodeStatus::SelfHealing => "▲ SELF-HEAL",
                    };

                    let prefix = if idx == 0 {
                        "  "
                    } else {
                        "  └──► "
                    };

                    out.push_str(&format!(
                        "{}[{}] {} [{}]\n",
                        prefix,
                        node.node_type.to_uppercase(),
                        node.label,
                        status_badge
                    ));
                }
            }
        } else {
            out.push_str("  (No nodes scheduled)\n");
        }

        out.push_str("└────────────────────────────────────────────────────────┘\n");
        out
    }
}
