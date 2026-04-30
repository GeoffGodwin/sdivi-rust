use sdi_core::compute::coupling::compute_coupling_topology;
use sdi_core::input::{DependencyGraphInput, EdgeInput, NodeInput};

fn node(id: &str) -> NodeInput {
    NodeInput { id: id.to_string(), path: id.to_string(), language: "rust".to_string() }
}

fn edge(src: &str, tgt: &str) -> EdgeInput {
    EdgeInput { source: src.to_string(), target: tgt.to_string() }
}

#[test]
fn empty_graph_returns_zeros() {
    let g = DependencyGraphInput { nodes: vec![], edges: vec![] };
    let r = compute_coupling_topology(&g).unwrap();
    assert_eq!(r.node_count, 0);
    assert_eq!(r.edge_count, 0);
    assert_eq!(r.density, 0.0);
    assert_eq!(r.cycle_count, 0);
    assert_eq!(r.component_count, 0);
    assert!(r.top_hubs.is_empty());
}

#[test]
fn two_node_graph_one_edge() {
    let g = DependencyGraphInput {
        nodes: vec![node("src/a.rs"), node("src/b.rs")],
        edges: vec![edge("src/a.rs", "src/b.rs")],
    };
    let r = compute_coupling_topology(&g).unwrap();
    assert_eq!(r.node_count, 2);
    assert_eq!(r.edge_count, 1);
    assert_eq!(r.component_count, 1);
    assert_eq!(r.cycle_count, 0);
}

#[test]
fn cycle_detected() {
    let g = DependencyGraphInput {
        nodes: vec![node("a.rs"), node("b.rs")],
        edges: vec![edge("a.rs", "b.rs"), edge("b.rs", "a.rs")],
    };
    let r = compute_coupling_topology(&g).unwrap();
    assert!(r.cycle_count > 0);
}

#[test]
fn disconnected_graph_has_multiple_components() {
    let g = DependencyGraphInput {
        nodes: vec![node("a.rs"), node("b.rs"), node("c.rs")],
        edges: vec![edge("a.rs", "b.rs")],
    };
    let r = compute_coupling_topology(&g).unwrap();
    assert_eq!(r.component_count, 2);
}

#[test]
fn invalid_node_id_returns_error() {
    let g = DependencyGraphInput {
        nodes: vec![NodeInput {
            id: "./bad".to_string(),
            path: "./bad".to_string(),
            language: "rust".to_string(),
        }],
        edges: vec![],
    };
    assert!(compute_coupling_topology(&g).is_err());
}

#[test]
fn self_loop_excluded_from_edge_count() {
    let g = DependencyGraphInput {
        nodes: vec![node("a.rs")],
        edges: vec![edge("a.rs", "a.rs")],
    };
    let r = compute_coupling_topology(&g).unwrap();
    assert_eq!(r.edge_count, 0);
}
