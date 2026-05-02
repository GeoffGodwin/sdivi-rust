use sdivi_snapshot::PatternMetricsResult;
use std::collections::BTreeMap;

use sdivi_config::{BoundaryDef, BoundarySpec};
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

fn make_boundary(name: &str) -> BoundaryDef {
    BoundaryDef {
        name: name.to_string(),
        description: None,
        modules: vec![format!("src/{}/**", name)],
        allow_imports_from: vec![],
    }
}

/// assemble_snapshot with Some(BoundarySpec) sets intent_divergence to Some.
#[test]
fn with_boundary_spec_intent_divergence_is_some() {
    let spec = BoundarySpec {
        version: None,
        boundaries: vec![make_boundary("api"), make_boundary("models")],
    };

    let snap = assemble_snapshot(
        empty_graph(),
        empty_partition(),
        PatternCatalog::default(),
        PatternMetricsResult::default(),
        Some(&spec),
        "2026-04-29T00:00:00Z",
        None,
        None,
    );

    assert!(
        snap.intent_divergence.is_some(),
        "intent_divergence must be Some when BoundarySpec is provided"
    );
}

/// boundary_count in intent_divergence matches the number of boundaries in the spec.
#[test]
fn boundary_count_matches_spec_length() {
    let spec = BoundarySpec {
        version: None,
        boundaries: vec![
            make_boundary("api"),
            make_boundary("models"),
            make_boundary("infra"),
        ],
    };

    let snap = assemble_snapshot(
        empty_graph(),
        empty_partition(),
        PatternCatalog::default(),
        PatternMetricsResult::default(),
        Some(&spec),
        "2026-04-29T00:00:00Z",
        None,
        None,
    );

    let info = snap.intent_divergence.unwrap();
    assert_eq!(
        info.boundary_count, 3,
        "boundary_count must equal the number of BoundaryDef entries"
    );
}

/// violation_count is always 0 (detection pass not yet implemented).
#[test]
fn violation_count_is_zero() {
    let spec = BoundarySpec {
        version: None,
        boundaries: vec![make_boundary("core"), make_boundary("ui")],
    };

    let snap = assemble_snapshot(
        empty_graph(),
        empty_partition(),
        PatternCatalog::default(),
        PatternMetricsResult::default(),
        Some(&spec),
        "2026-04-29T00:00:00Z",
        None,
        None,
    );

    let info = snap.intent_divergence.unwrap();
    assert_eq!(
        info.violation_count, 0,
        "violation_count must be 0 until the detection pass is implemented"
    );
}

/// An empty BoundarySpec (zero boundaries) still sets intent_divergence to Some with boundary_count 0.
#[test]
fn empty_boundary_spec_sets_intent_divergence_with_zero_count() {
    let spec = BoundarySpec {
        version: None,
        boundaries: vec![],
    };

    let snap = assemble_snapshot(
        empty_graph(),
        empty_partition(),
        PatternCatalog::default(),
        PatternMetricsResult::default(),
        Some(&spec),
        "2026-04-29T00:00:00Z",
        None,
        None,
    );

    let info = snap
        .intent_divergence
        .expect("intent_divergence must be Some even for empty spec");
    assert_eq!(
        info.boundary_count, 0,
        "boundary_count must be 0 for an empty spec"
    );
}

/// intent_divergence is serialized as present (not skipped) when boundary spec is given.
#[test]
fn intent_divergence_present_in_json_when_spec_given() {
    let spec = BoundarySpec {
        version: None,
        boundaries: vec![make_boundary("api")],
    };

    let snap = assemble_snapshot(
        empty_graph(),
        empty_partition(),
        PatternCatalog::default(),
        PatternMetricsResult::default(),
        Some(&spec),
        "2026-04-29T00:00:00Z",
        None,
        None,
    );

    let json = serde_json::to_string(&snap).unwrap();
    assert!(
        json.contains("\"intent_divergence\""),
        "intent_divergence must appear in JSON when boundary spec is present, got: {json}"
    );
    assert!(
        json.contains("\"boundary_count\""),
        "boundary_count must appear in JSON, got: {json}"
    );
}

/// intent_divergence is omitted from JSON when no boundary spec is given (Rule 16).
#[test]
fn intent_divergence_absent_from_json_when_no_spec() {
    let snap = assemble_snapshot(
        empty_graph(),
        empty_partition(),
        PatternCatalog::default(),
        PatternMetricsResult::default(),
        None,
        "2026-04-29T00:00:00Z",
        None,
        None,
    );

    let json = serde_json::to_string(&snap).unwrap();
    assert!(
        !json.contains("\"intent_divergence\""),
        "intent_divergence must be absent from JSON when no spec is given, got: {json}"
    );
}
