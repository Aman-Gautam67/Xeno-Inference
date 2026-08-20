//! Comprehensive integration test suite for `crates/xeno-dag`.

use xeno_dag::prelude::*;

#[tokio::test]
async fn test_dag_subgraph_grafting() {
    let mut main_graph = XenoDAGGraph::new();

    let root = XenoDAGNode::new("root", "Plan Task", "planner");
    main_graph.add_node(root).unwrap();

    // Create a dynamic sub-graph (e.g. self-healing branch)
    let mut sub_graph = XenoDAGGraph::new();
    let heal1 = XenoDAGNode::new("heal_patch", "Synthesize Patch", "coder");
    let heal2 = XenoDAGNode::new("heal_verify", "Verify Patch", "tester").with_dependencies(vec!["heal_patch".into()]);
    sub_graph.add_node(heal1).unwrap();
    sub_graph.add_node(heal2).unwrap();

    let grafted = main_graph.graft_subgraph("root", sub_graph).unwrap();
    assert_eq!(grafted.len(), 2);
    assert_eq!(main_graph.node_count(), 3);

    // Verify dependencies
    let heal_patch_node = main_graph.get_node("heal_patch").unwrap();
    assert!(heal_patch_node.dependencies.contains(&"root".to_string()));
}

#[tokio::test]
async fn test_dag_event_broadcast_stream() {
    let mut graph = XenoDAGGraph::new();
    let mut rx = graph.subscribe();

    let node = XenoDAGNode::new("test_node", "Test Task", "coder");
    graph.add_node(node).unwrap();

    let ev1 = rx.recv().await.unwrap();
    assert_eq!(ev1.node_id, "test_node");
    assert_eq!(ev1.event_type, DAGEventType::NodeAdded);

    graph.update_status("test_node", NodeStatus::Running).unwrap();
    let ev2 = rx.recv().await.unwrap();
    assert_eq!(ev2.status, NodeStatus::Running);
    assert_eq!(ev2.event_type, DAGEventType::StatusChanged);

    graph.update_status("test_node", NodeStatus::Success).unwrap();
    let ev3 = rx.recv().await.unwrap();
    assert_eq!(ev3.status, NodeStatus::Success);
}

#[tokio::test]
async fn test_dag_topological_execution_order() {
    let mut graph = XenoDAGGraph::new();

    let n1 = XenoDAGNode::new("1", "A", "t");
    let n2 = XenoDAGNode::new("2", "B", "t").with_dependencies(vec!["1".into()]);
    let n3 = XenoDAGNode::new("3", "C", "t").with_dependencies(vec!["1".into()]);
    let n4 = XenoDAGNode::new("4", "D", "t").with_dependencies(vec!["2".into(), "3".into()]);

    graph.add_node(n1).unwrap();
    graph.add_node(n2).unwrap();
    graph.add_node(n3).unwrap();
    graph.add_node(n4).unwrap();

    let order = graph.topological_sort().unwrap();
    assert_eq!(order.len(), 4);
    assert_eq!(order[0], "1");
    assert_eq!(order[3], "4");
}
