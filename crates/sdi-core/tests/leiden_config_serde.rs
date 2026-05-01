//! Serde round-trip tests for LeidenConfigInput, including the edge_weights field.
//!
//! The `edge_weights` field is `Option<BTreeMap<(String, String), f64>>`.
//! serde_json cannot serialize a map whose keys are tuples — they serialize as
//! JSON arrays, which are not valid JSON object keys.  This test surfaces that
//! known limitation.

use std::collections::BTreeMap;

use sdi_core::input::{LeidenConfigInput, QualityFunctionInput};

#[test]
fn leiden_config_default_round_trips_through_serde_json() {
    let cfg = LeidenConfigInput::default();
    let value = serde_json::to_value(&cfg).expect("LeidenConfigInput::default() must serialize");
    let back: LeidenConfigInput =
        serde_json::from_value(value).expect("must deserialize back");
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
    };
    let value = serde_json::to_value(&cfg).expect("None edge_weights must serialize");
    let back: LeidenConfigInput = serde_json::from_value(value).expect("must deserialize back");
    assert_eq!(back, cfg);
}

#[test]
fn leiden_config_populated_edge_weights_round_trips_through_serde_json() {
    // serde_json requires map keys to be strings; BTreeMap<(String, String), f64>
    // serializes tuple keys as JSON arrays, not strings — this is a known
    // limitation (flagged as a non-blocking reviewer note in M15).
    // This test documents the actual behavior: serialization must succeed.
    let mut weights = BTreeMap::new();
    weights.insert(("src/a.rs".to_string(), "src/b.rs".to_string()), 3.0_f64);
    weights.insert(("src/a.rs".to_string(), "src/c.rs".to_string()), 1.5_f64);
    let cfg = LeidenConfigInput {
        seed: 42,
        gamma: 1.0,
        iterations: 100,
        quality: QualityFunctionInput::Modularity,
        edge_weights: Some(weights.clone()),
    };
    let value = serde_json::to_value(&cfg)
        .expect("LeidenConfigInput with populated edge_weights must serialize without error");
    let back: LeidenConfigInput =
        serde_json::from_value(value).expect("must deserialize back to LeidenConfigInput");
    assert_eq!(back.seed, cfg.seed);
    assert_eq!(back.edge_weights, cfg.edge_weights);
}
