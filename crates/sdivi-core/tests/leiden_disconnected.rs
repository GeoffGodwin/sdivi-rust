use sdivi_core::compute::boundaries::detect_boundaries;
use sdivi_core::input::{DependencyGraphInput, EdgeInput, LeidenConfigInput, NodeInput};

fn node(id: &str) -> NodeInput {
    NodeInput {
        id: id.to_string(),
        path: id.to_string(),
        language: "rust".to_string(),
    }
}

fn edge(src: &str, tgt: &str) -> EdgeInput {
    EdgeInput {
        source: src.to_string(),
        target: tgt.to_string(),
    }
}

/// Three fully disconnected components of 4 nodes each → at least 3 communities.
#[test]
fn three_disconnected_components_detected() {
    let nodes: Vec<NodeInput> = (0..12).map(|i| node(&format!("n{i}.rs"))).collect();
    // Three cliques of 4 with no inter-component edges.
    let edges: Vec<EdgeInput> = (0..3usize)
        .flat_map(|c| {
            let base = c * 4;
            vec![
                edge(&format!("n{}.rs", base), &format!("n{}.rs", base + 1)),
                edge(&format!("n{}.rs", base + 1), &format!("n{}.rs", base + 2)),
                edge(&format!("n{}.rs", base + 2), &format!("n{}.rs", base + 3)),
                edge(&format!("n{}.rs", base + 3), &format!("n{}.rs", base)),
            ]
        })
        .collect();

    let g = DependencyGraphInput { nodes, edges };
    let r = detect_boundaries(&g, &LeidenConfigInput::default(), &[]).unwrap();
    assert_eq!(
        r.disconnected_components, 3,
        "expected 3 disconnected components"
    );
    assert!(
        r.community_count >= 3,
        "should detect at least 3 communities"
    );
}

/// Empty graph → 0 communities, 0 disconnected_components.
#[test]
fn empty_graph_has_no_components() {
    let g = DependencyGraphInput {
        nodes: vec![],
        edges: vec![],
    };
    let r = detect_boundaries(&g, &LeidenConfigInput::default(), &[]).unwrap();
    assert_eq!(r.community_count, 0);
    assert_eq!(r.disconnected_components, 0);
    assert_eq!(r.historical_stability, 0.0);
}

/// Single node → 1 community, 1 component.
#[test]
fn single_node_one_community() {
    let g = DependencyGraphInput {
        nodes: vec![node("solo.rs")],
        edges: vec![],
    };
    let r = detect_boundaries(&g, &LeidenConfigInput::default(), &[]).unwrap();
    assert_eq!(r.disconnected_components, 1);
    assert_eq!(r.community_count, 1);
}

/// Fully connected graph → 1 component.
#[test]
fn connected_graph_one_component() {
    let g = DependencyGraphInput {
        nodes: vec![node("a.rs"), node("b.rs"), node("c.rs")],
        edges: vec![
            edge("a.rs", "b.rs"),
            edge("b.rs", "c.rs"),
            edge("c.rs", "a.rs"),
        ],
    };
    let r = detect_boundaries(&g, &LeidenConfigInput::default(), &[]).unwrap();
    assert_eq!(r.disconnected_components, 1);
}
