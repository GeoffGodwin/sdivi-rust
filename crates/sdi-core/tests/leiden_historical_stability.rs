use std::collections::BTreeMap;

use sdi_core::compute::boundaries::detect_boundaries;
use sdi_core::input::{DependencyGraphInput, EdgeInput, LeidenConfigInput, NodeInput, PriorPartition};

fn node(id: &str) -> NodeInput {
    NodeInput { id: id.to_string(), path: id.to_string(), language: "rust".to_string() }
}

fn edge(src: &str, tgt: &str) -> EdgeInput {
    EdgeInput { source: src.to_string(), target: tgt.to_string() }
}

fn stable_graph() -> DependencyGraphInput {
    DependencyGraphInput {
        nodes: vec![node("a.rs"), node("b.rs"), node("c.rs"), node("d.rs")],
        edges: vec![
            edge("a.rs", "b.rs"), edge("b.rs", "a.rs"),
            edge("c.rs", "d.rs"), edge("d.rs", "c.rs"),
        ],
    }
}

/// Empty prior → historical_stability is 0.0 (no pairs to compare).
#[test]
fn empty_prior_returns_zero() {
    let g = stable_graph();
    let r = detect_boundaries(&g, &LeidenConfigInput::default(), &[]).unwrap();
    assert_eq!(r.historical_stability, 0.0);
}

/// Single prior entry → still 0.0 (need at least 2 for a pair comparison).
#[test]
fn single_prior_returns_zero() {
    let g = stable_graph();
    let mut assignments = BTreeMap::new();
    assignments.insert("a.rs".to_string(), 0u32);
    assignments.insert("b.rs".to_string(), 0u32);
    assignments.insert("c.rs".to_string(), 1u32);
    assignments.insert("d.rs".to_string(), 1u32);
    let prior = vec![PriorPartition { cluster_assignments: assignments }];
    let r = detect_boundaries(&g, &LeidenConfigInput::default(), &prior).unwrap();
    assert_eq!(r.historical_stability, 0.0);
}

/// Two identical priors → 1 pair that agrees → stability = 1.0.
#[test]
fn two_identical_priors_stability_one() {
    let g = stable_graph();
    let mut a = BTreeMap::new();
    a.insert("a.rs".to_string(), 0u32);
    a.insert("b.rs".to_string(), 0u32);
    a.insert("c.rs".to_string(), 1u32);
    a.insert("d.rs".to_string(), 1u32);
    let prior = vec![
        PriorPartition { cluster_assignments: a.clone() },
        PriorPartition { cluster_assignments: a },
    ];
    let r = detect_boundaries(&g, &LeidenConfigInput::default(), &prior).unwrap();
    assert!((r.historical_stability - 1.0).abs() < 1e-10);
}

/// Two priors with different node-set membership → 0 pairs agree → stability = 0.0.
///
/// Note: stability is node-set-based, not ID-based. Renaming community IDs while
/// keeping the same node sets still counts as matching. To get non-matching pairs,
/// we need to change which nodes are grouped together.
#[test]
fn two_different_priors_stability_zero() {
    let g = stable_graph();
    // Prior a: {a,b} in comm 0; {c,d} in comm 1.
    let mut a = BTreeMap::new();
    a.insert("a.rs".to_string(), 0u32);
    a.insert("b.rs".to_string(), 0u32);
    a.insert("c.rs".to_string(), 1u32);
    a.insert("d.rs".to_string(), 1u32);
    // Prior b: {a,c} in comm 0; {b,d} in comm 1. — completely different membership.
    let mut b = BTreeMap::new();
    b.insert("a.rs".to_string(), 0u32);
    b.insert("b.rs".to_string(), 1u32);
    b.insert("c.rs".to_string(), 0u32);
    b.insert("d.rs".to_string(), 1u32);
    let prior = vec![
        PriorPartition { cluster_assignments: a },
        PriorPartition { cluster_assignments: b },
    ];
    let r = detect_boundaries(&g, &LeidenConfigInput::default(), &prior).unwrap();
    assert!((r.historical_stability - 0.0).abs() < 1e-10);
}

/// Stability is in [0, 1] regardless of input shape.
#[test]
fn stability_in_range() {
    let g = stable_graph();
    let make_partition = |swap: bool| -> PriorPartition {
        let mut m = BTreeMap::new();
        if swap {
            m.insert("a.rs".to_string(), 1u32);
            m.insert("b.rs".to_string(), 1u32);
            m.insert("c.rs".to_string(), 0u32);
            m.insert("d.rs".to_string(), 0u32);
        } else {
            m.insert("a.rs".to_string(), 0u32);
            m.insert("b.rs".to_string(), 0u32);
            m.insert("c.rs".to_string(), 1u32);
            m.insert("d.rs".to_string(), 1u32);
        }
        PriorPartition { cluster_assignments: m }
    };
    let prior = vec![make_partition(false), make_partition(true), make_partition(false)];
    let r = detect_boundaries(&g, &LeidenConfigInput::default(), &prior).unwrap();
    assert!(r.historical_stability >= 0.0 && r.historical_stability <= 1.0);
}
