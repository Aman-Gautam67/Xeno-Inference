//! Execution DAG engine with topological ordering, dynamic subgraph grafting, and broadcast streaming.

use crate::events::{DAGEventType, DAGNodeEvent};
use crate::node::{NodeStatus, XenoDAGNode};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;
use tokio::sync::broadcast;

/// Errors arising from DAG graph manipulations.
#[derive(Debug, Error)]
pub enum DAGError {
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Cycle detected in DAG: dependency cycle involving node {0}")]
    CycleDetected(String),

    #[error("Node already exists: {0}")]
    NodeAlreadyExists(String),

    #[error("Invalid status transition from {from:?} to {to:?} for node {node_id}")]
    InvalidStatusTransition {
        node_id: String,
        from: NodeStatus,
        to: NodeStatus,
    },
}

/// Dynamic Directed Acyclic Graph state tracker.
#[derive(Debug)]
pub struct XenoDAGGraph {
    nodes: HashMap<String, XenoDAGNode>,
    /// Outgoing edges: from_id -> list of to_ids
    outgoing_edges: HashMap<String, Vec<String>>,
    /// Incoming edges: to_id -> list of from_ids
    incoming_edges: HashMap<String, Vec<String>>,
    event_tx: broadcast::Sender<DAGNodeEvent>,
}

impl Default for XenoDAGGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl XenoDAGGraph {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            nodes: HashMap::new(),
            outgoing_edges: HashMap::new(),
            incoming_edges: HashMap::new(),
            event_tx,
        }
    }

    /// Inserts a new node into the graph.
    pub fn add_node(&mut self, node: XenoDAGNode) -> Result<String, DAGError> {
        let node_id = node.node_id.clone();
        if self.nodes.contains_key(&node_id) {
            return Err(DAGError::NodeAlreadyExists(node_id));
        }

        // Register declared dependencies
        for dep in &node.dependencies {
            self.outgoing_edges
                .entry(dep.clone())
                .or_default()
                .push(node_id.clone());
            self.incoming_edges
                .entry(node_id.clone())
                .or_default()
                .push(dep.clone());
        }

        let event = DAGNodeEvent::new(
            &node_id,
            DAGEventType::NodeAdded,
            node.status,
            format!("Node '{}' added to execution DAG", node.label),
        );
        let _ = self.event_tx.send(event);

        self.nodes.insert(node_id.clone(), node);
        Ok(node_id)
    }

    /// Adds an execution dependency edge: `from_id` must finish before `to_id` can run.
    pub fn add_edge(&mut self, from_id: &str, to_id: &str) -> Result<(), DAGError> {
        if !self.nodes.contains_key(from_id) {
            return Err(DAGError::NodeNotFound(from_id.to_string()));
        }
        if !self.nodes.contains_key(to_id) {
            return Err(DAGError::NodeNotFound(to_id.to_string()));
        }

        self.outgoing_edges
            .entry(from_id.to_string())
            .or_default()
            .push(to_id.to_string());
        self.incoming_edges
            .entry(to_id.to_string())
            .or_default()
            .push(from_id.to_string());

        if let Some(target_node) = self.nodes.get_mut(to_id) {
            if !target_node.dependencies.contains(&from_id.to_string()) {
                target_node.dependencies.push(from_id.to_string());
            }
        }

        if self.has_cycles() {
            return Err(DAGError::CycleDetected(to_id.to_string()));
        }

        Ok(())
    }

    /// Updates node status and broadcasts transition event.
    pub fn update_status(&mut self, node_id: &str, status: NodeStatus) -> Result<(), DAGError> {
        let node = self
            .nodes
            .get_mut(node_id)
            .ok_or_else(|| DAGError::NodeNotFound(node_id.to_string()))?;

        let old_status = node.status;
        node.status = status;

        let event = DAGNodeEvent::new(
            node_id,
            DAGEventType::StatusChanged,
            status,
            format!("Status changed from {old_status:?} to {status:?}"),
        );
        let _ = self.event_tx.send(event);

        Ok(())
    }

    /// Attaches output payload to a node.
    pub fn set_output_payload(
        &mut self,
        node_id: &str,
        payload: serde_json::Value,
    ) -> Result<(), DAGError> {
        let node = self
            .nodes
            .get_mut(node_id)
            .ok_or_else(|| DAGError::NodeNotFound(node_id.to_string()))?;

        node.output_payload = Some(payload.clone());

        let event = DAGNodeEvent::new(
            node_id,
            DAGEventType::OutputAttached,
            node.status,
            "Output payload attached",
        )
        .with_payload(payload);
        let _ = self.event_tx.send(event);

        Ok(())
    }

    /// Returns all nodes in `Pending` whose dependencies are all in `Success`.
    pub fn get_ready_nodes(&self) -> Vec<String> {
        let mut ready = Vec::new();

        for (id, node) in &self.nodes {
            if node.status != NodeStatus::Pending {
                continue;
            }

            let deps = self.incoming_edges.get(id);
            let all_deps_succeeded = match deps {
                None => true,
                Some(dep_list) => dep_list.iter().all(|d_id| {
                    self.nodes
                        .get(d_id)
                        .is_some_and(|d_node| d_node.status == NodeStatus::Success)
                }),
            };

            if all_deps_succeeded {
                ready.push(id.clone());
            }
        }

        ready
    }

    /// Dynamically grafts a subgraph attaching its root nodes to `parent_id`.
    pub fn graft_subgraph(
        &mut self,
        parent_id: &str,
        subgraph: XenoDAGGraph,
    ) -> Result<Vec<String>, DAGError> {
        if !self.nodes.contains_key(parent_id) {
            return Err(DAGError::NodeNotFound(parent_id.to_string()));
        }

        let mut grafted_ids = Vec::new();

        for (_, mut node) in subgraph.nodes {
            let n_id = node.node_id.clone();
            if node.dependencies.is_empty() {
                node.dependencies.push(parent_id.to_string());
            }
            self.add_node(node)?;
            grafted_ids.push(n_id);
        }

        for (from, targets) in subgraph.outgoing_edges {
            for to in targets {
                let _ = self.add_edge(&from, &to);
            }
        }

        let event = DAGNodeEvent::new(
            parent_id,
            DAGEventType::SubgraphGrafted,
            NodeStatus::Running,
            format!("Grafted {} dynamic nodes under '{parent_id}'", grafted_ids.len()),
        );
        let _ = self.event_tx.send(event);

        Ok(grafted_ids)
    }

    /// Detects if the graph contains any directed cycles using Kahn's algorithm.
    pub fn has_cycles(&self) -> bool {
        self.topological_sort().is_err()
    }

    /// Performs a topological sort of all node IDs.
    pub fn topological_sort(&self) -> Result<Vec<String>, DAGError> {
        let mut in_degrees: HashMap<String, usize> = HashMap::new();
        for id in self.nodes.keys() {
            let in_deg = self.incoming_edges.get(id).map_or(0, |v| v.len());
            in_degrees.insert(id.clone(), in_deg);
        }

        let mut queue: VecDeque<String> = in_degrees
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut order = Vec::new();

        while let Some(curr) = queue.pop_front() {
            order.push(curr.clone());

            if let Some(children) = self.outgoing_edges.get(&curr) {
                for child in children {
                    if let Some(deg) = in_degrees.get_mut(child) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(child.clone());
                        }
                    }
                }
            }
        }

        if order.len() != self.nodes.len() {
            Err(DAGError::CycleDetected("Cycle detected".into()))
        } else {
            Ok(order)
        }
    }

    /// Subscribes to real-time DAG mutation and transition events.
    pub fn subscribe(&self) -> broadcast::Receiver<DAGNodeEvent> {
        self.event_tx.subscribe()
    }

    /// Retrieves an immutable reference to a node.
    pub fn get_node(&self, node_id: &str) -> Option<&XenoDAGNode> {
        self.nodes.get(node_id)
    }

    /// Returns the total number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns true if all nodes are in terminal state (`Success` or `Failed`).
    pub fn is_all_completed(&self) -> bool {
        self.nodes
            .values()
            .all(|n| matches!(n.status, NodeStatus::Success | NodeStatus::Failed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dag_lifecycle_and_ready_nodes() {
        let mut graph = XenoDAGGraph::new();

        let n1 = XenoDAGNode::new("n1", "Plan", "planner");
        let n2 = XenoDAGNode::new("n2", "Code", "coder").with_dependencies(vec!["n1".into()]);
        let n3 = XenoDAGNode::new("n3", "Test", "tester").with_dependencies(vec!["n2".into()]);

        graph.add_node(n1).unwrap();
        graph.add_node(n2).unwrap();
        graph.add_node(n3).unwrap();

        assert_eq!(graph.get_ready_nodes(), vec!["n1"]);

        graph.update_status("n1", NodeStatus::Running).unwrap();
        assert!(graph.get_ready_nodes().is_empty());

        graph.update_status("n1", NodeStatus::Success).unwrap();
        assert_eq!(graph.get_ready_nodes(), vec!["n2"]);

        graph.update_status("n2", NodeStatus::Success).unwrap();
        assert_eq!(graph.get_ready_nodes(), vec!["n3"]);
    }

    #[test]
    fn test_dag_cycle_detection() {
        let mut graph = XenoDAGGraph::new();
        let n1 = XenoDAGNode::new("a", "A", "task");
        let n2 = XenoDAGNode::new("b", "B", "task").with_dependencies(vec!["a".into()]);

        graph.add_node(n1).unwrap();
        graph.add_node(n2).unwrap();

        // Adding reverse edge creates cycle
        let cycle_err = graph.add_edge("b", "a");
        assert!(cycle_err.is_err());
    }
}
