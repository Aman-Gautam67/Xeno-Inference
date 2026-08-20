//! XENO INFERENCE — Desktop Spatial Canvas Tauri Backend (`xeno-tauri`).
//!
//! Provides IPC commands and state management bridging Tauri v2 with `xeno-core`,
//! `xeno-dag`, and `xeno-agent` for the React 19 spatial canvas workspace.

use serde::{Deserialize, Serialize};

/// Serialized canvas node structure for WebGL / React projection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasNodeProjection {
    pub id: String,
    pub node_type: String,
    pub position: CanvasPosition,
    pub data: serde_json::Value,
}

/// 2D coordinate on the infinite spatial canvas.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CanvasPosition {
    pub x: f64,
    pub y: f64,
}

/// System state payload sent over Tauri IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TauriWorkspaceState {
    pub project_name: String,
    pub active_nodes: Vec<CanvasNodeProjection>,
    pub vram_used_bytes: u64,
    pub token_velocity: f64,
    pub total_cost_usd: f64,
}

impl Default for TauriWorkspaceState {
    fn default() -> Self {
        Self {
            project_name: "XENO INFERENCE".into(),
            active_nodes: vec![
                CanvasNodeProjection {
                    id: "prompt-1".into(),
                    node_type: "prompt_block".into(),
                    position: CanvasPosition { x: 100.0, y: 250.0 },
                    data: serde_json::json!({
                        "label": "User Instruction",
                        "content": "Implement AST validation engine",
                        "status": "completed"
                    }),
                },
            ],
            vram_used_bytes: 14 * 1024 * 1024 * 1024,
            token_velocity: 138.4,
            total_cost_usd: 0.0182,
        }
    }
}

/// IPC command returning current canvas workspace state.
pub fn get_workspace_state() -> TauriWorkspaceState {
    TauriWorkspaceState::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tauri_workspace_state_serialization() {
        let state = get_workspace_state();
        assert_eq!(state.project_name, "XENO INFERENCE");
        assert_eq!(state.active_nodes.len(), 1);
        assert_eq!(state.active_nodes[0].node_type, "prompt_block");

        let serialized = serde_json::to_string(&state).unwrap();
        assert!(serialized.contains("prompt_block"));
    }
}
