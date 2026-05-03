//! Tests for `assemble_snapshot`'s `boundary_count` parameter and the
//! resulting `intent_divergence` field.
//!
//! As of v0.2.0, `assemble_snapshot` takes `Option<usize>` (the boundary
//! count) directly rather than `Option<&BoundarySpec>`; the caller derives
//! the count from whatever boundary representation it owns.

use sdivi_snapshot::PatternMetricsResult;
use std::collections::BTreeMap;

use sdivi_detection::partition::LeidenPartition;
use sdivi_graph::metrics::GraphMetrics;
use sdivi_patterns::PatternCatalog;
use sdivi_snapshot::assemble_snapshot;

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

/// assemble_snapshot with Some(count) sets intent_divergence to Some.
#[test]
fn with_boundary_count_intent_divergence_is_some() {
    let snap = assemble_snapshot(
        empty_graph(),
        empty_partition(),
        PatternCatalog::default(),
        PatternMetricsResult::default(),
        Some(2),
        "2026-04-29T00:00:00Z",
        None,
        None,
        0,
    );

    assert!(
        snap.intent_divergence.is_some(),
        "intent_divergence must be Some when boundary_count is provided"
    );
}

/// boundary_count in intent_divergence matches the value passed to assemble_snapshot.
#[test]
fn boundary_count_threads_through() {
    let snap = assemble_snapshot(
        empty_graph(),
        empty_partition(),
        PatternCatalog::default(),
        PatternMetricsResult::default(),
        Some(3),
        "2026-04-29T00:00:00Z",
        None,
        None,
        0,
    );

    let info = snap.intent_divergence.unwrap();
    assert_eq!(
        info.boundary_count, 3,
        "boundary_count must match the value passed to assemble_snapshot"
    );
}

/// violation_count in intent_divergence matches the value passed to assemble_snapshot.
#[test]
fn violation_count_threads_through() {
    let snap = assemble_snapshot(
        empty_graph(),
        empty_partition(),
        PatternCatalog::default(),
        PatternMetricsResult::default(),
        Some(2),
        "2026-04-29T00:00:00Z",
        None,
        None,
        7,
    );

    let info = snap.intent_divergence.unwrap();
    assert_eq!(
        info.violation_count, 7,
        "violation_count must match the value passed to assemble_snapshot"
    );
}

/// A boundary_count of zero still sets intent_divergence to Some with boundary_count 0.
#[test]
fn zero_boundary_count_sets_intent_divergence_with_zero_count() {
    let snap = assemble_snapshot(
        empty_graph(),
        empty_partition(),
        PatternCatalog::default(),
        PatternMetricsResult::default(),
        Some(0),
        "2026-04-29T00:00:00Z",
        None,
        None,
        0,
    );

    let info = snap
        .intent_divergence
        .expect("intent_divergence must be Some even when boundary_count is 0");
    assert_eq!(
        info.boundary_count, 0,
        "boundary_count must be 0 when caller passed Some(0)"
    );
}

/// intent_divergence is serialized as present (not skipped) when boundary_count is given.
#[test]
fn intent_divergence_present_in_json_when_count_given() {
    let snap = assemble_snapshot(
        empty_graph(),
        empty_partition(),
        PatternCatalog::default(),
        PatternMetricsResult::default(),
        Some(1),
        "2026-04-29T00:00:00Z",
        None,
        None,
        0,
    );

    let json = serde_json::to_string(&snap).unwrap();
    assert!(
        json.contains("\"intent_divergence\""),
        "intent_divergence must appear in JSON when boundary_count is present, got: {json}"
    );
    assert!(
        json.contains("\"boundary_count\""),
        "boundary_count must appear in JSON, got: {json}"
    );
}

/// intent_divergence is omitted from JSON when no boundary_count is given (Rule 16).
#[test]
fn intent_divergence_absent_from_json_when_no_count() {
    let snap = assemble_snapshot(
        empty_graph(),
        empty_partition(),
        PatternCatalog::default(),
        PatternMetricsResult::default(),
        None,
        "2026-04-29T00:00:00Z",
        None,
        None,
        0,
    );

    let json = serde_json::to_string(&snap).unwrap();
    assert!(
        !json.contains("\"intent_divergence\""),
        "intent_divergence must be absent from JSON when no count is given, got: {json}"
    );
}
