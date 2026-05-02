//! WASM smoke tests — run via `wasm-pack test --node`.
//!
//! Each test verifies that an exported function is callable and produces
//! non-trivial output.  Cross-platform hash determinism for `normalize_and_hash`
//! is also verified here (same fixture → same 64-char hex on all platforms).

use sdivi_wasm::types::{
    WasmAssembleSnapshotInput, WasmBoundaryDefInput, WasmBoundarySpecInput,
    WasmDependencyGraphInput, WasmDivergenceSummary, WasmEdgeInput, WasmLeidenConfigInput,
    WasmNodeInput, WasmPatternInstanceInput, WasmPatternMetricsResult, WasmPriorPartition,
    WasmQualityFunction, WasmSnapshotPriorPartition, WasmThresholdsInput,
};
use sdivi_wasm::{
    assemble_snapshot, compute_boundary_violations, compute_coupling_topology, compute_delta,
    compute_pattern_metrics, compute_thresholds_check, compute_trend, detect_boundaries,
    infer_boundaries, normalize_and_hash,
};
use serde_wasm_bindgen;
use wasm_bindgen_test::wasm_bindgen_test;

// `wasm-pack test --node` selects Node as the test runner; that's also the
// default when no `wasm_bindgen_test_configure!` macro call is present.
// We don't call the macro because the explicit `run_in_node` token isn't
// recognised by the wasm-bindgen-test version compatible with our
// rustc 1.85 / wasm-bindgen 0.2.117 pin (it expects `run_in_browser` only).

fn two_node_graph() -> WasmDependencyGraphInput {
    WasmDependencyGraphInput {
        nodes: vec![
            WasmNodeInput {
                id: "src/lib.rs".into(),
                path: "src/lib.rs".into(),
                language: "rust".into(),
            },
            WasmNodeInput {
                id: "src/models.rs".into(),
                path: "src/models.rs".into(),
                language: "rust".into(),
            },
        ],
        edges: vec![WasmEdgeInput {
            source: "src/lib.rs".into(),
            target: "src/models.rs".into(),
        }],
    }
}

fn default_leiden_cfg() -> WasmLeidenConfigInput {
    WasmLeidenConfigInput {
        seed: 42,
        gamma: 1.0,
        iterations: 100,
        quality: WasmQualityFunction::Modularity,
    }
}

#[wasm_bindgen_test]
fn test_compute_coupling_topology_returns_correct_counts() {
    let result = compute_coupling_topology(two_node_graph()).unwrap();
    assert_eq!(result.node_count, 2);
    assert_eq!(result.edge_count, 1);
    assert!(result.density > 0.0);
}

#[wasm_bindgen_test]
fn test_detect_boundaries_returns_assignments() {
    let result = detect_boundaries(two_node_graph(), default_leiden_cfg(), vec![]).unwrap();
    assert_eq!(result.cluster_assignments.len(), 2);
    assert!(result.community_count >= 1);
}

#[wasm_bindgen_test]
fn test_compute_boundary_violations_empty_spec() {
    let spec = WasmBoundarySpecInput { boundaries: vec![] };
    let result = compute_boundary_violations(two_node_graph(), spec).unwrap();
    assert_eq!(result.violation_count, 0);
}

#[wasm_bindgen_test]
fn test_compute_boundary_violations_with_spec() {
    let spec = WasmBoundarySpecInput {
        boundaries: vec![
            WasmBoundaryDefInput {
                name: "core".into(),
                modules: vec!["src/lib.rs".into()],
                allow_imports_from: vec![],
            },
            WasmBoundaryDefInput {
                name: "data".into(),
                modules: vec!["src/models.rs".into()],
                allow_imports_from: vec!["core".into()],
            },
        ],
    };
    let result = compute_boundary_violations(two_node_graph(), spec).unwrap();
    // lib→models is allowed (data allows imports from core; no restriction on core outgoing).
    assert_eq!(result.violation_count, 0);
}

#[wasm_bindgen_test]
fn test_compute_pattern_metrics_empty() {
    let result = compute_pattern_metrics(vec![]).unwrap();
    assert_eq!(result.total_entropy, 0.0);
    assert_eq!(result.convention_drift, 0.0);
}

#[wasm_bindgen_test]
fn test_compute_pattern_metrics_non_trivial() {
    let patterns = vec![
        WasmPatternInstanceInput {
            fingerprint: "a".repeat(64),
            category: "error_handling".into(),
            node_id: "src/lib.rs".into(),
            location: None,
        },
        WasmPatternInstanceInput {
            fingerprint: "b".repeat(64),
            category: "error_handling".into(),
            node_id: "src/models.rs".into(),
            location: None,
        },
    ];
    let result = compute_pattern_metrics(patterns).unwrap();
    // Two distinct fingerprints in same category → H = 1.0 bit
    assert!((result.entropy_per_category["error_handling"] - 1.0).abs() < 1e-9);
    assert!(result.total_entropy > 0.0);
}

#[wasm_bindgen_test]
fn test_compute_thresholds_check_no_breach() {
    let summary = WasmDivergenceSummary {
        pattern_entropy_delta: Some(0.5),
        convention_drift_delta: Some(0.1),
        coupling_delta: Some(0.05),
        community_count_delta: Some(0),
        boundary_violation_delta: None,
        pattern_entropy_per_category_delta: None,
        convention_drift_per_category_delta: None,
    };
    let cfg = WasmThresholdsInput {
        pattern_entropy_rate: 2.0,
        convention_drift_rate: 3.0,
        coupling_delta_rate: 0.15,
        boundary_violation_rate: 2.0,
        overrides: Default::default(),
        today: "2026-05-01".into(),
    };
    let result = compute_thresholds_check(summary, cfg).unwrap();
    assert!(!result.breached);
    assert!(result.breaches.is_empty());
}

#[wasm_bindgen_test]
fn test_compute_thresholds_check_breach() {
    let summary = WasmDivergenceSummary {
        pattern_entropy_delta: Some(5.0),
        ..Default::default()
    };
    let cfg = WasmThresholdsInput {
        pattern_entropy_rate: 2.0,
        convention_drift_rate: 3.0,
        coupling_delta_rate: 0.15,
        boundary_violation_rate: 2.0,
        overrides: Default::default(),
        today: "2026-05-01".into(),
    };
    let result = compute_thresholds_check(summary, cfg).unwrap();
    assert!(result.breached);
    assert_eq!(result.breaches[0].dimension, "pattern_entropy");
}

#[wasm_bindgen_test]
fn test_infer_boundaries_empty() {
    let result = infer_boundaries(vec![], 3).unwrap();
    assert!(result.proposals.is_empty());
    assert_eq!(result.partition_count, 0);
}

#[wasm_bindgen_test]
fn test_infer_boundaries_with_stable_community() {
    // infer_boundaries takes WasmSnapshotPriorPartition (the snapshot-shaped
    // version with a `partition_id` field), not the leiden-flavoured
    // WasmPriorPartition used by detect_boundaries. Distinct types are
    // intentional — see the comment above WasmSnapshotPriorPartition.
    let mut assignments = std::collections::BTreeMap::new();
    assignments.insert("src/lib.rs".to_string(), 0u32);
    assignments.insert("src/models.rs".to_string(), 1u32);
    let p = WasmSnapshotPriorPartition {
        cluster_assignments: assignments.clone(),
    };
    let result = infer_boundaries(vec![p.clone(), p.clone(), p], 2).unwrap();
    assert!(!result.proposals.is_empty());
}

/// normalize_and_hash must produce the same 64-char hex on all platforms.
#[wasm_bindgen_test]
fn test_normalize_and_hash_deterministic() {
    let h1 = normalize_and_hash("try_expression", vec![]).unwrap();
    let h2 = normalize_and_hash("try_expression", vec![]).unwrap();
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 64);
    assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
}

/// The hash must differ for different node kinds.
#[wasm_bindgen_test]
fn test_normalize_and_hash_differs_by_kind() {
    let a = normalize_and_hash("try_expression", vec![]).unwrap();
    let b = normalize_and_hash("match_expression", vec![]).unwrap();
    assert_ne!(a, b);
}

/// Dedicated test for CI cross-platform hash determinism check.
/// Prints hash in CI_HASH format so CI can grep and compare across platforms.
#[wasm_bindgen_test]
fn normalize_hash_deterministic() {
    let hash = normalize_and_hash("try_expression", vec![]).unwrap();
    println!("CI_HASH:{}", hash);
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

// ── assemble_snapshot / compute_delta / compute_trend coverage ────────────────

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
    // The returned JsValue must be a valid Snapshot: compute_delta must accept it.
    // Comparing a snapshot to itself must produce exactly-zero coupling delta.
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

    // Build a JS array from the three Snapshot JsValues.
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
    // All other slopes must also be present (not None) for a 3-snapshot window.
    assert!(result.pattern_entropy_slope.is_some());
    assert!(result.convention_drift_slope.is_some());
    assert!(result.community_count_slope.is_some());
}

// ── ADL-4 & ADL-7 Verification Tests ────────────────────────────────────────

/// ADL-4 verification: WasmLeidenConfigInput must NOT have an `edge_weights` field.
/// This is an intentional MVP gap documented in the ARCHITECTURE_LOG.md.
/// WASM bindings expose unweighted Leiden only.
#[wasm_bindgen_test]
fn test_adl4_wasm_leiden_config_input_omits_edge_weights() {
    // Create a config with all fields that are present
    let config = WasmLeidenConfigInput {
        seed: 42,
        gamma: 1.0,
        iterations: 100,
        quality: WasmQualityFunction::Modularity,
    };

    // Verify that config can be created without edge_weights field
    // (if edge_weights were required, this would not compile)
    assert_eq!(config.seed, 42);
    assert_eq!(config.gamma, 1.0);
    assert_eq!(config.iterations, 100);

    // The absence of edge_weights is verified by type system at compile time
    // This test confirms the struct shape matches the WASM-only surface
}

/// ADL-7 verification: assemble_snapshot must hardcode change_coupling to None in MVP.
/// This is an intentional MVP limitation documented in the ARCHITECTURE_LOG.md.
/// Post-MVP, expose `compute_change_coupling` in WASM and add field to input.
#[wasm_bindgen_test]
fn test_adl7_assemble_snapshot_change_coupling_is_none() {
    let snap_js = assemble_snapshot(make_assemble_input(0.25, "2026-05-01T00:00:00Z")).unwrap();
    let snap: sdivi_core::Snapshot =
        serde_wasm_bindgen::from_value(snap_js).expect("must deserialize as Snapshot");

    // Verify that change_coupling is None (MVP limitation)
    assert!(
        snap.change_coupling.is_none(),
        "change_coupling must be None in MVP (ADL-7 — hardcoded to None)"
    );
}
