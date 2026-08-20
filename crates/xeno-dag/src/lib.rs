//! XENO INFERENCE — Real-Time Execution DAG Engine (`xeno-dag`).
//!
//! Provides petgraph-compatible directed acyclic graph execution state tracking,
//! topological task ordering, cycle detection, dynamic subgraph grafting,
//! and broadcast event streaming for user interfaces.

pub mod events;
pub mod graph;
pub mod node;

pub use events::{DAGEventType, DAGNodeEvent};
pub use graph::{DAGError, XenoDAGGraph};
pub use node::{AssignedModel, NodeStatus, XenoDAGNode};

/// Prelude exporting all DAG primitives.
pub mod prelude {
    pub use super::events::{DAGEventType, DAGNodeEvent};
    pub use super::graph::{DAGError, XenoDAGGraph};
    pub use super::node::{AssignedModel, NodeStatus, XenoDAGNode};
}
