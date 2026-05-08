//! Serde round-trip tests for LeidenConfigInput, including the edge_weights field.
//!
//! `edge_weights` uses `BTreeMap<String, f64>` with NUL-delimited string keys
//! (`"source\x00target"`) so that `serde_json` can serialize them as JSON
//! object keys. Use [`sdivi_core::input::edge_weight_key`] to construct keys.

use std::collections::BTreeMap;

use sdivi_core::compute::boundaries::detect_boundaries;
use sdivi_core::input::{
    edge_weight_key, DependencyGraphInput, EdgeInput, LeidenConfigInput, NodeInput,
    QualityFunctionInput,
};

#[test]
fn leiden_config_default_round_trips_through_serde_json() {
    let cfg = LeidenConfigInput::default();
    let value = serde_json::to_value(&cfg).expect("LeidenConfigInput::default() must serialize");
    let back: LeidenConfigInput = serde_json::from_value(value).expect("must deserialize back");
    assert_eq!(back, cfg);
}

#[test]
fn leiden_config_none_edge_weights_round_trips() {
    let cfg = LeidenConfigInput {
        seed: 7,
        gamma: 0.5,
        iterations: 50,
        quality: QualityFunctionInput::Cpm,
        edge_weights: None,
        min_compression_ratio: 0.1,
        max_recursion_depth: 32,
    };
    let value = serde_json::to_value(&cfg).expect("None edge_weights must serialize");
    let back: LeidenConfigInput = serde_json::from_value(value).expect("must deserialize back");
    assert_eq!(back, cfg);
}

#[test]
fn leiden_config_populated_edge_weights_round_trips_through_serde_json() {
    let mut weights = BTreeMap::new();
    weights.insert(edge_weight_key("src/a.rs", "src/b.rs"), 3.0_f64);
    weights.insert(edge_weight_key("src/a.rs", "src/c.rs"), 1.5_f64);
    let cfg = LeidenConfigInput {
        seed: 42,
        gamma: 1.0,
        iterations: 100,
        quality: QualityFunctionInput::Modularity,
        edge_weights: Some(weights.clone()),
        min_compression_ratio: 0.1,
        max_recursion_depth: 32,
    };
    let value = serde_json::to_value(&cfg)
        .expect("LeidenConfigInput with populated edge_weights must serialize");
    let back: LeidenConfigInput =
        serde_json::from_value(value).expect("must deserialize back to LeidenConfigInput");
    assert_eq!(back.seed, cfg.seed);
    assert_eq!(back.edge_weights, cfg.edge_weights);
}

#[test]
fn detect_boundaries_normalizes_wrong_order_edge_weight_keys() {
    // Test the normalization fallback at boundaries.rs:109:
    // if si < ti { (si, ti) } else { (ti, si) }
    // When edge weights keys are provided in reverse order (target < source),
    // the normalization should still apply them correctly.

    // Create a simple 3-node graph: a -> b, a -> c, b -> c
    let graph = DependencyGraphInput {
        nodes: vec![
            NodeInput {
                id: "a".to_string(),
                path: "a".to_string(),
                language: "rust".to_string(),
            },
            NodeInput {
                id: "b".to_string(),
                path: "b".to_string(),
                language: "rust".to_string(),
            },
            NodeInput {
                id: "c".to_string(),
                path: "c".to_string(),
                language: "rust".to_string(),
            },
        ],
        edges: vec![
            EdgeInput {
                source: "a".to_string(),
                target: "b".to_string(),
            },
            EdgeInput {
                source: "a".to_string(),
                target: "c".to_string(),
            },
            EdgeInput {
                source: "b".to_string(),
                target: "c".to_string(),
            },
        ],
    };

    // Create edge weights with at least one key in wrong order: b > a
    // The normalization should reorder it internally to (a, b).
    let mut weights = BTreeMap::new();
    // Correct order: a < b
    weights.insert(edge_weight_key("a", "b"), 2.0_f64);
    // Wrong order: c > b, will be normalized to (b, c)
    weights.insert(edge_weight_key("c", "b"), 1.5_f64);

    let cfg = LeidenConfigInput {
        seed: 42,
        gamma: 1.0,
        iterations: 50,
        quality: QualityFunctionInput::Modularity,
        edge_weights: Some(weights),
        min_compression_ratio: 0.1,
        max_recursion_depth: 32,
    };

    let result = detect_boundaries(&graph, &cfg, &[]).expect("detect_boundaries should succeed");

    // Verify that the result contains all three nodes in their communities
    assert_eq!(result.cluster_assignments.len(), 3);
    assert!(result.cluster_assignments.contains_key("a"));
    assert!(result.cluster_assignments.contains_key("b"));
    assert!(result.cluster_assignments.contains_key("c"));
}
