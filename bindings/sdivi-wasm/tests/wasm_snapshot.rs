//! WASM snapshot assembly / delta / trend tests, plus M22 change-coupling tests.
//!
//! Run via `wasm-pack test --node`.

use sdivi_wasm::change_coupling::{WasmChangeCouplingConfigInput, WasmCoChangeEventInput};
use sdivi_wasm::types::{
    WasmAssembleSnapshotInput, WasmChangeCouplingInput, WasmCoChangePairInput,
    WasmLeidenConfigInput, WasmPatternMetricsResult, WasmQualityFunction,
};
use sdivi_wasm::{assemble_snapshot, compute_change_coupling, compute_delta, compute_trend};
use serde_wasm_bindgen;
use wasm_bindgen_test::wasm_bindgen_test;

/// Build a minimal [`WasmAssembleSnapshotInput`] with the given graph density.
///
/// Uses two nodes in a single community so `build_leiden_partition` has valid
/// community-ID strings to parse.
fn make_assemble_input(density: f64, timestamp: &str) -> WasmAssembleSnapshotInput {
    let mut cluster_assignments = std::collections::BTreeMap::new();
    cluster_assignments.insert("src/lib.rs".into(), 0u32);
    cluster_assignments.insert("src/models.rs".into(), 0u32);

    let mut internal_edge_density = std::collections::BTreeMap::new();
    internal_edge_density.insert("0".into(), 1.0f64);

    WasmAssembleSnapshotInput {
        node_ids: vec!["src/lib.rs".into(), "src/models.rs".into()],
        cluster_assignments,
        internal_edge_density,
        modularity: 0.0,
        node_count: 2,
        edge_count: 1,
        density,
        cycle_count: 0,
        top_hubs: vec![],
        component_count: 1,
        pattern_metrics: WasmPatternMetricsResult {
            entropy_per_category: std::collections::BTreeMap::new(),
            total_entropy: 0.0,
            convention_drift: 0.0,
            convention_drift_per_category: std::collections::BTreeMap::new(),
        },
        pattern_instances: vec![],
        timestamp: timestamp.into(),
        commit: None,
        boundary_count: None,
        leiden_seed: Some(42),
        violation_count: None,
        change_coupling: None,
    }
}

/// `assemble_snapshot` with `violation_count` set produces a snapshot whose
/// `intent_divergence` field carries the expected counts.
#[wasm_bindgen_test]
fn test_assemble_snapshot_with_violation_count_sets_intent_divergence() {
    let mut input = make_assemble_input(0.25, "2026-05-01T00:00:00Z");
    input.boundary_count = Some(3);
    input.violation_count = Some(5);
    let snap_js = assemble_snapshot(input).unwrap();
    assert!(!snap_js.is_null());
    let snap: sdivi_core::Snapshot =
        serde_wasm_bindgen::from_value(snap_js).expect("must deserialize as Snapshot");
    let id = snap
        .intent_divergence
        .expect("intent_divergence must be Some when boundary_count is set");
    assert_eq!(id.boundary_count, 3, "boundary_count must match input");
    assert_eq!(id.violation_count, 5, "violation_count must match input");
}

/// `assemble_snapshot` returns a non-null JS object that `compute_delta` can
/// accept, and comparing a snapshot to itself yields an exactly-zero coupling
/// delta — proving the round-trip serialization is correct.
#[wasm_bindgen_test]
fn test_assemble_snapshot_produces_valid_snapshot_json() {
    let snap_js = assemble_snapshot(make_assemble_input(0.25, "2026-05-01T00:00:00Z")).unwrap();
    assert!(!snap_js.is_null());
    assert!(!snap_js.is_undefined());
    let delta = compute_delta(snap_js.clone(), snap_js).unwrap();
    assert_eq!(
        delta.coupling_delta,
        Some(0.0),
        "self-delta coupling must be zero (density unchanged)"
    );
}

/// `compute_delta` with two snapshots that differ in graph density must produce
/// a non-zero `coupling_delta` equal to the density difference.
#[wasm_bindgen_test]
fn test_compute_delta_distinct_snapshots_produces_nonzero_coupling_delta() {
    let snap1_js = assemble_snapshot(make_assemble_input(0.1, "2026-04-01T00:00:00Z")).unwrap();
    let snap2_js = assemble_snapshot(make_assemble_input(0.3, "2026-05-01T00:00:00Z")).unwrap();
    let delta = compute_delta(snap1_js, snap2_js).unwrap();
    let cd = delta
        .coupling_delta
        .expect("coupling_delta must be Some when both snapshots have density");
    assert!(
        (cd - 0.2).abs() < 1e-9,
        "expected coupling_delta ≈ 0.2 (density 0.1→0.3), got {cd}"
    );
}

/// `compute_trend` with three snapshots of steadily increasing density must
/// return non-None slopes with a coupling slope equal to the per-interval mean.
#[wasm_bindgen_test]
fn test_compute_trend_with_multiple_snapshots_returns_nonzero_slopes() {
    let snap1 = assemble_snapshot(make_assemble_input(0.1, "2026-03-01T00:00:00Z")).unwrap();
    let snap2 = assemble_snapshot(make_assemble_input(0.3, "2026-04-01T00:00:00Z")).unwrap();
    let snap3 = assemble_snapshot(make_assemble_input(0.5, "2026-05-01T00:00:00Z")).unwrap();

    let arr = js_sys::Array::new();
    arr.push(&snap1);
    arr.push(&snap2);
    arr.push(&snap3);

    let result = compute_trend(arr.into(), None).unwrap();
    assert_eq!(result.snapshot_count, 3);
    let slope = result
        .coupling_slope
        .expect("coupling_slope must be Some for 3 snapshots");
    // Each interval adds 0.2 density; mean slope = 0.2.
    assert!(
        (slope - 0.2).abs() < 1e-9,
        "expected coupling slope ≈ 0.2 (constant increment 0.2/snapshot), got {slope}"
    );
    assert!(result.pattern_entropy_slope.is_some());
    assert!(result.convention_drift_slope.is_some());
    assert!(result.community_count_slope.is_some());
}

// ── ADL-4 implemented (M21) & M22 change-coupling tests ─────────────────────

/// M21: WasmLeidenConfigInput now has an optional `edge_weights` field.
/// Unweighted callers pass `edge_weights: None`; the type system prevents
/// accidental use of weighted partitions in code that doesn't supply weights.
#[wasm_bindgen_test]
fn test_m21_wasm_leiden_config_input_edge_weights_optional() {
    let config = WasmLeidenConfigInput {
        seed: 42,
        gamma: 1.0,
        iterations: 100,
        quality: WasmQualityFunction::Modularity,
        edge_weights: None,
    };
    assert_eq!(config.seed, 42);
    assert!(config.edge_weights.is_none());
}

/// M22: omitting `change_coupling` produces a snapshot with `change_coupling: None`.
/// This is the backward-compatible path — existing callers see no change.
#[wasm_bindgen_test]
fn test_assemble_snapshot_without_change_coupling_produces_none() {
    let snap_js = assemble_snapshot(make_assemble_input(0.25, "2026-05-01T00:00:00Z")).unwrap();
    let snap: sdivi_core::Snapshot =
        serde_wasm_bindgen::from_value(snap_js).expect("must deserialize as Snapshot");
    assert!(
        snap.change_coupling.is_none(),
        "change_coupling must be None when omitted from WasmAssembleSnapshotInput"
    );
}

/// M22: passing `change_coupling: Some(...)` populates the assembled snapshot field.
/// The round-trip uses compute_change_coupling output directly.
#[wasm_bindgen_test]
fn test_assemble_snapshot_with_change_coupling_round_trips() {
    let events = vec![
        WasmCoChangeEventInput {
            commit_sha: "abc".into(),
            commit_date: "2026-01-01T00:00:00Z".into(),
            files: vec!["src/a.rs".into(), "src/b.rs".into()],
        },
        WasmCoChangeEventInput {
            commit_sha: "def".into(),
            commit_date: "2026-01-02T00:00:00Z".into(),
            files: vec!["src/a.rs".into(), "src/b.rs".into()],
        },
    ];
    let cfg = WasmChangeCouplingConfigInput {
        min_frequency: 0.5,
        history_depth: 10,
    };
    let cc_result = compute_change_coupling(events, cfg).unwrap();

    let mut input = make_assemble_input(0.25, "2026-05-01T00:00:00Z");
    input.change_coupling = Some(WasmChangeCouplingInput {
        pairs: cc_result.pairs.iter().map(|p| WasmCoChangePairInput {
            source: p.source.clone(),
            target: p.target.clone(),
            frequency: p.frequency,
            cochange_count: p.cochange_count,
        }).collect(),
        commits_analyzed: cc_result.commits_analyzed,
        distinct_files_touched: cc_result.distinct_files_touched,
    });

    let snap_js = assemble_snapshot(input).unwrap();
    let snap: sdivi_core::Snapshot =
        serde_wasm_bindgen::from_value(snap_js).expect("must deserialize as Snapshot");

    let cc = snap
        .change_coupling
        .expect("change_coupling must be Some when supplied");
    assert_eq!(cc.commits_analyzed, 2, "commits_analyzed must round-trip");
    assert_eq!(cc.distinct_files_touched, 2, "distinct_files_touched must round-trip");
    assert_eq!(cc.pairs.len(), 1, "one pair above min_frequency expected");
    assert_eq!(cc.pairs[0].source, "src/a.rs");
    assert_eq!(cc.pairs[0].target, "src/b.rs");
    assert!(
        (cc.pairs[0].frequency - 1.0).abs() < 1e-9,
        "frequency must be 1.0 (both commits touched both files)"
    );
}
