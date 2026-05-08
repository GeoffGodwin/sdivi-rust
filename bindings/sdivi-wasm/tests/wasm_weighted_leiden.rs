//! Tests for weighted Leiden community detection via WASM — M21.
//!
//! Run via `wasm-pack test --node bindings/sdivi-wasm`.

use std::collections::BTreeMap;

use sdivi_wasm::detect_boundaries;
use sdivi_wasm::types::{
    WasmDependencyGraphInput, WasmEdgeInput, WasmLeidenConfigInput, WasmNodeInput,
    WasmQualityFunction,
};
use wasm_bindgen_test::wasm_bindgen_test;

// ── Graph fixture ─────────────────────────────────────────────────────────────

/// Four-node graph with edges (a,b), (a,c), (b,c), (c,d).
/// Triangle a-b-c plus a tail edge c→d.
fn four_node_graph() -> WasmDependencyGraphInput {
    WasmDependencyGraphInput {
        nodes: vec![
            WasmNodeInput {
                id: "a".into(),
                path: "a".into(),
                language: "rust".into(),
            },
            WasmNodeInput {
                id: "b".into(),
                path: "b".into(),
                language: "rust".into(),
            },
            WasmNodeInput {
                id: "c".into(),
                path: "c".into(),
                language: "rust".into(),
            },
            WasmNodeInput {
                id: "d".into(),
                path: "d".into(),
                language: "rust".into(),
            },
        ],
        edges: vec![
            WasmEdgeInput {
                source: "a".into(),
                target: "b".into(),
            },
            WasmEdgeInput {
                source: "a".into(),
                target: "c".into(),
            },
            WasmEdgeInput {
                source: "b".into(),
                target: "c".into(),
            },
            WasmEdgeInput {
                source: "c".into(),
                target: "d".into(),
            },
        ],
    }
}

fn unweighted_cfg() -> WasmLeidenConfigInput {
    WasmLeidenConfigInput {
        seed: 42,
        gamma: 1.0,
        iterations: 100,
        quality: WasmQualityFunction::Modularity,
        edge_weights: None,
        min_compression_ratio: 0.1,
        max_recursion_depth: 32,
    }
}

fn weighted_cfg(weights: BTreeMap<String, f64>) -> WasmLeidenConfigInput {
    WasmLeidenConfigInput {
        seed: 42,
        gamma: 1.0,
        iterations: 100,
        quality: WasmQualityFunction::Modularity,
        edge_weights: Some(weights),
        min_compression_ratio: 0.1,
        max_recursion_depth: 32,
    }
}

// ── Acceptance tests ──────────────────────────────────────────────────────────

/// Weighted run with dominant weights on the cross-cut edges (a,c) and (b,c)
/// produces a different partition than the unweighted run on the same graph.
#[wasm_bindgen_test]
fn test_detect_boundaries_weighted_differs_from_unweighted() {
    let graph = four_node_graph();

    let unweighted = detect_boundaries(graph.clone(), unweighted_cfg(), vec![]).unwrap();

    // Weight the edges that *cross* the unweighted {a,b} | {c,d} partition.
    // Reinforcing edges already inside a community is a no-op; pulling on
    // cross-cut edges is what actually moves nodes between communities.
    let mut weights = BTreeMap::new();
    weights.insert("a:c".to_string(), 100.0);
    weights.insert("b:c".to_string(), 100.0);
    let weighted = detect_boundaries(graph, weighted_cfg(weights), vec![]).unwrap();

    assert_ne!(
        unweighted.cluster_assignments, weighted.cluster_assignments,
        "weighted Leiden should produce a different partition than unweighted on this graph"
    );
}

/// A key with no colon is rejected with a clear JsError.
#[wasm_bindgen_test]
fn test_detect_boundaries_rejects_malformed_weight_key() {
    let mut weights = BTreeMap::new();
    weights.insert("a-b".to_string(), 1.0);
    let result = detect_boundaries(four_node_graph(), weighted_cfg(weights), vec![]);
    let e = result.unwrap_err();
    let msg = format!("{:?}", e);
    assert!(
        msg.contains("no colon found") || msg.contains("not in"),
        "error message should name the problem: {msg}"
    );
}

/// A negative weight is rejected.
#[wasm_bindgen_test]
fn test_detect_boundaries_rejects_negative_weight() {
    let mut weights = BTreeMap::new();
    weights.insert("a:b".to_string(), -0.5);
    let result = detect_boundaries(four_node_graph(), weighted_cfg(weights), vec![]);
    let e = result.unwrap_err();
    let msg = format!("{:?}", e);
    assert!(
        msg.contains("negative") || msg.contains(">= 0.0"),
        "error message should mention negative weight: {msg}"
    );
}

/// A NaN weight is rejected.
#[wasm_bindgen_test]
fn test_detect_boundaries_rejects_nan_weight() {
    let mut weights = BTreeMap::new();
    weights.insert("a:b".to_string(), f64::NAN);
    let result = detect_boundaries(four_node_graph(), weighted_cfg(weights), vec![]);
    let e = result.unwrap_err();
    let msg = format!("{:?}", e);
    assert!(
        msg.contains("NaN") || msg.contains("finite"),
        "error message should mention NaN: {msg}"
    );
}

/// Zero weight is accepted (treated as unweighted for that edge).
#[wasm_bindgen_test]
fn test_detect_boundaries_accepts_zero_weight() {
    let mut weights = BTreeMap::new();
    weights.insert("a:b".to_string(), 0.0);
    let result = detect_boundaries(four_node_graph(), weighted_cfg(weights), vec![]);
    assert!(result.is_ok(), "zero weight should be accepted");
}

/// Same seed + same weights → bit-identical partition across two runs (determinism).
#[wasm_bindgen_test]
fn test_detect_boundaries_weighted_deterministic() {
    let mut weights = BTreeMap::new();
    weights.insert("a:b".to_string(), 10.0);
    weights.insert("c:d".to_string(), 10.0);

    let r1 = detect_boundaries(four_node_graph(), weighted_cfg(weights.clone()), vec![]).unwrap();
    let r2 = detect_boundaries(four_node_graph(), weighted_cfg(weights), vec![]).unwrap();

    assert_eq!(
        r1.cluster_assignments, r2.cluster_assignments,
        "same seed + same weights must produce identical partitions"
    );
    assert_eq!(r1.modularity, r2.modularity);
}
