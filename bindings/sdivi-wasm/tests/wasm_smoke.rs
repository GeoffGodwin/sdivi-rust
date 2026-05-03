//! WASM smoke tests — run via `wasm-pack test --node`.
//!
//! Each test verifies that an exported function is callable and produces
//! non-trivial output.  Cross-platform hash determinism for `normalize_and_hash`
//! is also verified here (same fixture → same 64-char hex on all platforms).
//! Snapshot assembly / delta / trend tests live in `wasm_snapshot.rs`.

use sdivi_wasm::types::{
    WasmBoundaryDefInput, WasmBoundarySpecInput, WasmDependencyGraphInput, WasmDivergenceSummary,
    WasmEdgeInput, WasmLeidenConfigInput, WasmNodeInput, WasmPatternInstanceInput,
    WasmQualityFunction, WasmSnapshotPriorPartition, WasmThresholdsInput,
};
use sdivi_wasm::{
    compute_boundary_violations, compute_coupling_topology, compute_pattern_metrics,
    compute_thresholds_check, detect_boundaries, infer_boundaries, normalize_and_hash,
};
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
fn test_compute_boundary_violations_allowed_import() {
    // core → data is allowed because core.allow_imports_from = ["data"].
    let spec = WasmBoundarySpecInput {
        boundaries: vec![
            WasmBoundaryDefInput {
                name: "core".into(),
                modules: vec!["src/lib.rs".into()],
                allow_imports_from: vec!["data".into()],
            },
            WasmBoundaryDefInput {
                name: "data".into(),
                modules: vec!["src/models.rs".into()],
                allow_imports_from: vec![],
            },
        ],
    };
    let result = compute_boundary_violations(two_node_graph(), spec).unwrap();
    assert_eq!(result.violation_count, 0);
}

#[wasm_bindgen_test]
fn test_compute_boundary_violations_disallowed_import() {
    // core → data is a violation because core.allow_imports_from is empty.
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
    assert_eq!(result.violation_count, 1);
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
