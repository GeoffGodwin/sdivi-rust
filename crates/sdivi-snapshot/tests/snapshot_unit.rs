//! Unit tests for `assemble_snapshot` and `Snapshot` serde behaviour.
//! Moved here from the `snapshot.rs` inline `#[cfg(test)]` block so that
//! `snapshot.rs` stays under the 300-line ceiling.

use std::collections::BTreeMap;

use sdivi_detection::partition::LeidenPartition;
use sdivi_graph::metrics::GraphMetrics;
use sdivi_patterns::PatternCatalog;
use sdivi_snapshot::snapshot::{
    assemble_snapshot, PatternMetricsResult, Snapshot, SNAPSHOT_VERSION,
};

fn empty_graph() -> GraphMetrics {
    GraphMetrics {
        node_count: 0,
        edge_count: 0,
        density: 0.0,
        cycle_count: 0,
        top_hubs: vec![],
        component_count: 0,
    }
}

fn empty_partition() -> LeidenPartition {
    LeidenPartition {
        assignments: BTreeMap::new(),
        stability: BTreeMap::new(),
        modularity: 0.0,
        seed: 42,
    }
}

fn make_snap() -> Snapshot {
    assemble_snapshot(
        empty_graph(),
        empty_partition(),
        PatternCatalog::default(),
        PatternMetricsResult::default(),
        None,
        "T",
        None,
        None,
        0,
    )
}

#[test]
fn assemble_snapshot_sets_version() {
    let snap = make_snap();
    assert_eq!(snap.snapshot_version, SNAPSHOT_VERSION);
}

#[test]
fn no_boundary_spec_means_no_intent_divergence() {
    let snap = make_snap();
    assert!(snap.intent_divergence.is_none());
}

#[test]
fn commit_round_trips() {
    let snap = assemble_snapshot(
        empty_graph(),
        empty_partition(),
        PatternCatalog::default(),
        PatternMetricsResult::default(),
        None,
        "T",
        Some("deadbeef"),
        None,
        0,
    );
    assert_eq!(snap.commit.as_deref(), Some("deadbeef"));
}

#[test]
fn serde_round_trip() {
    let snap = make_snap();
    let json = serde_json::to_string(&snap).unwrap();
    let decoded: Snapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(snap, decoded);
}

#[test]
fn commit_none_absent_from_json() {
    let snap = make_snap();
    let json = serde_json::to_string(&snap).unwrap();
    assert!(!json.contains("\"commit\""));
}

#[test]
fn pattern_metrics_present_in_json() {
    let snap = make_snap();
    let json = serde_json::to_string(&snap).unwrap();
    assert!(json.contains("\"pattern_metrics\""));
}
