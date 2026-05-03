//! Native (non-wasm) tests for M22 types — run with `cargo test -p sdivi-wasm --test m22_native`.
//!
//! These complement the wasm_bindgen_test suite in wasm_snapshot.rs (which needs wasm-pack)
//! by covering the serde field-name contract and struct accessibility without a WASM runtime.

use sdivi_wasm::types::{WasmChangeCouplingInput, WasmCoChangePairInput};

// ── WasmCoChangePairInput ─────────────────────────────────────────────────────

#[test]
fn wasm_co_change_pair_input_fields_survive_serde_round_trip() {
    let pair = WasmCoChangePairInput {
        source: "src/a.rs".into(),
        target: "src/b.rs".into(),
        frequency: 0.75,
        cochange_count: 3,
    };
    let json = serde_json::to_value(&pair).unwrap();
    let back: WasmCoChangePairInput = serde_json::from_value(json).unwrap();
    assert_eq!(back.source, "src/a.rs");
    assert_eq!(back.target, "src/b.rs");
    assert!((back.frequency - 0.75).abs() < 1e-10, "frequency must survive round-trip");
    assert_eq!(back.cochange_count, 3);
}

/// The JSON field names of `WasmCoChangePairInput` must match `sdivi_core::CoChangePair`
/// exactly — `exports::assemble_snapshot` converts between the two via a serde_json round-trip.
#[test]
fn wasm_co_change_pair_input_json_field_names_match_sdivi_core_contract() {
    let pair = WasmCoChangePairInput {
        source: "x.rs".into(),
        target: "y.rs".into(),
        frequency: 0.5,
        cochange_count: 2,
    };
    let json = serde_json::to_value(&pair).unwrap();
    assert!(
        json.get("source").is_some(),
        "field 'source' must be present in JSON (must match sdivi_core::CoChangePair)"
    );
    assert!(
        json.get("target").is_some(),
        "field 'target' must be present in JSON (must match sdivi_core::CoChangePair)"
    );
    assert!(
        json.get("frequency").is_some(),
        "field 'frequency' must be present in JSON (must match sdivi_core::CoChangePair)"
    );
    assert!(
        json.get("cochange_count").is_some(),
        "field 'cochange_count' must be present in JSON (must match sdivi_core::CoChangePair)"
    );
    // No extra fields
    assert_eq!(
        json.as_object().unwrap().len(),
        4,
        "expected exactly 4 fields: source, target, frequency, cochange_count"
    );
}

// ── WasmChangeCouplingInput ───────────────────────────────────────────────────

#[test]
fn wasm_change_coupling_input_fields_survive_serde_round_trip() {
    let cc = WasmChangeCouplingInput {
        pairs: vec![WasmCoChangePairInput {
            source: "a.rs".into(),
            target: "b.rs".into(),
            frequency: 1.0,
            cochange_count: 5,
        }],
        commits_analyzed: 5,
        distinct_files_touched: 2,
    };
    let json = serde_json::to_value(&cc).unwrap();
    let back: WasmChangeCouplingInput = serde_json::from_value(json).unwrap();
    assert_eq!(back.commits_analyzed, 5);
    assert_eq!(back.distinct_files_touched, 2);
    assert_eq!(back.pairs.len(), 1);
    assert_eq!(back.pairs[0].source, "a.rs");
    assert_eq!(back.pairs[0].target, "b.rs");
    assert!((back.pairs[0].frequency - 1.0).abs() < 1e-10);
    assert_eq!(back.pairs[0].cochange_count, 5);
}

/// The JSON field names of `WasmChangeCouplingInput` must match `sdivi_core::ChangeCouplingResult`
/// exactly — `exports::assemble_snapshot` converts between the two via a serde_json round-trip.
#[test]
fn wasm_change_coupling_input_json_field_names_match_sdivi_core_contract() {
    let cc = WasmChangeCouplingInput {
        pairs: vec![],
        commits_analyzed: 4,
        distinct_files_touched: 3,
    };
    let json = serde_json::to_value(&cc).unwrap();
    assert!(
        json.get("pairs").is_some(),
        "field 'pairs' must be present (must match sdivi_core::ChangeCouplingResult)"
    );
    assert!(
        json.get("commits_analyzed").is_some(),
        "field 'commits_analyzed' must be present (must match sdivi_core::ChangeCouplingResult)"
    );
    assert!(
        json.get("distinct_files_touched").is_some(),
        "field 'distinct_files_touched' must be present (must match sdivi_core::ChangeCouplingResult)"
    );
    // No extra fields
    assert_eq!(
        json.as_object().unwrap().len(),
        3,
        "expected exactly 3 fields: pairs, commits_analyzed, distinct_files_touched"
    );
}

#[test]
fn wasm_change_coupling_input_empty_pairs_survives_serde_round_trip() {
    let cc = WasmChangeCouplingInput {
        pairs: vec![],
        commits_analyzed: 10,
        distinct_files_touched: 5,
    };
    let json = serde_json::to_value(&cc).unwrap();
    assert_eq!(
        json["pairs"].as_array().unwrap().len(),
        0,
        "empty pairs must serialize to an empty array"
    );
    let back: WasmChangeCouplingInput = serde_json::from_value(json).unwrap();
    assert_eq!(back.pairs.len(), 0);
    assert_eq!(back.commits_analyzed, 10);
    assert_eq!(back.distinct_files_touched, 5);
}

/// Verify multiple pairs round-trip correctly (more than one entry in the list).
#[test]
fn wasm_change_coupling_input_multiple_pairs_serde_round_trip() {
    let cc = WasmChangeCouplingInput {
        pairs: vec![
            WasmCoChangePairInput {
                source: "src/a.rs".into(),
                target: "src/b.rs".into(),
                frequency: 0.8,
                cochange_count: 4,
            },
            WasmCoChangePairInput {
                source: "src/c.rs".into(),
                target: "src/d.rs".into(),
                frequency: 0.6,
                cochange_count: 3,
            },
        ],
        commits_analyzed: 5,
        distinct_files_touched: 4,
    };
    let json = serde_json::to_value(&cc).unwrap();
    let back: WasmChangeCouplingInput = serde_json::from_value(json).unwrap();
    assert_eq!(back.pairs.len(), 2);
    assert_eq!(back.pairs[0].source, "src/a.rs");
    assert_eq!(back.pairs[1].source, "src/c.rs");
    assert_eq!(back.commits_analyzed, 5);
}
