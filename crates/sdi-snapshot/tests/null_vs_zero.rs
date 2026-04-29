use std::collections::BTreeMap;

use sdi_detection::partition::LeidenPartition;
use sdi_graph::metrics::GraphMetrics;
use sdi_patterns::PatternCatalog;
use sdi_snapshot::build_snapshot;
use sdi_snapshot::compute_delta;
use sdi_snapshot::null_summary;
use sdi_snapshot::Snapshot;

fn identical_snap() -> Snapshot {
    build_snapshot(
        GraphMetrics {
            node_count: 2,
            edge_count: 1,
            density: 0.5,
            cycle_count: 0,
            top_hubs: vec![],
            component_count: 1,
        },
        LeidenPartition {
            assignments: BTreeMap::new(),
            stability: BTreeMap::from([(0usize, 1.0f64), (1, 1.0)]),
            modularity: 0.3,
            seed: 42,
        },
        PatternCatalog::default(),
        None,
        "2026-04-29T00:00:00Z",
        None,
    )
}

/// `null_summary()` produces a DivergenceSummary with all fields None.
#[test]
fn first_snapshot_has_null_deltas() {
    let s = null_summary();
    assert!(s.pattern_entropy_delta.is_none());
    assert!(s.coupling_delta.is_none());
    assert!(s.community_count_delta.is_none());
    assert!(s.boundary_violation_delta.is_none());
}

/// `compute_delta` of a snapshot against itself yields all Some(0) / Some(0.0).
#[test]
fn identical_snapshots_have_zero_deltas() {
    let snap = identical_snap();
    let delta = compute_delta(&snap, &snap);

    assert_eq!(
        delta.coupling_delta,
        Some(0.0),
        "coupling_delta must be Some(0.0) for identical snapshots"
    );
    assert_eq!(
        delta.community_count_delta,
        Some(0),
        "community_count_delta must be Some(0) for identical snapshots"
    );
    assert_eq!(
        delta.pattern_entropy_delta,
        Some(0.0),
        "pattern_entropy_delta must be Some(0.0) for identical snapshots"
    );
    // boundary_violation_delta is None because neither snapshot has intent_divergence.
    assert!(delta.boundary_violation_delta.is_none());
}

/// Serializing null_summary() produces explicit JSON null for every field.
#[test]
fn null_summary_json_has_explicit_nulls() {
    let s = null_summary();
    let json = serde_json::to_string(&s).unwrap();

    assert!(
        json.contains("\"coupling_delta\":null"),
        "coupling_delta must be explicit null in JSON, got: {json}"
    );
    assert!(
        json.contains("\"pattern_entropy_delta\":null"),
        "pattern_entropy_delta must be explicit null in JSON, got: {json}"
    );
    assert!(
        json.contains("\"community_count_delta\":null"),
        "community_count_delta must be explicit null in JSON, got: {json}"
    );
    assert!(
        json.contains("\"boundary_violation_delta\":null"),
        "boundary_violation_delta must be explicit null in JSON, got: {json}"
    );
}

/// Serializing a zero delta produces `0.0` (not `null`) for numeric fields.
#[test]
fn zero_delta_not_null_in_json() {
    let snap = identical_snap();
    let delta = compute_delta(&snap, &snap);
    let json = serde_json::to_string(&delta).unwrap();

    assert!(
        !json.contains("\"coupling_delta\":null"),
        "coupling_delta must NOT be null for a zero delta, got: {json}"
    );
    assert!(
        json.contains("\"coupling_delta\":0.0") || json.contains("\"coupling_delta\":0"),
        "coupling_delta must be 0.0 for identical snapshots, got: {json}"
    );
}
