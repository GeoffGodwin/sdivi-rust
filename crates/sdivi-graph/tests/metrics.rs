//! Tests for dependency graph construction and metrics.
//!
//! Uses hand-built FeatureRecords (no tree-sitter parsing required) for fast,
//! deterministic tests. Includes a test asserting the hand-known node/edge
//! counts for the `simple-rust` fixture.

use sdivi_graph::dependency_graph::build_dependency_graph;
use sdivi_graph::metrics::compute_metrics;
use sdivi_parsing::feature_record::FeatureRecord;
use std::path::PathBuf;

fn make_record(path: &str, imports: &[&str]) -> FeatureRecord {
    FeatureRecord {
        path: PathBuf::from(path),
        language: "rust".to_string(),
        imports: imports.iter().map(|s| s.to_string()).collect(),
        exports: vec![],
        signatures: vec![],
        pattern_hints: vec![],
    }
}

// ── simple-rust fixture expected counts ─────────────────────────────────────

#[test]
fn simple_rust_fixture_five_nodes_zero_edges() {
    // The simple-rust fixture has 5 .rs files, all importing only from std or
    // serde (external). No intra-fixture imports via `use` statements exist, so
    // the resolved graph has 0 edges.
    let records = vec![
        make_record("src/lib.rs", &["std::collections::BTreeMap", "std::fmt"]),
        make_record("src/models.rs", &["serde::{Deserialize, Serialize}"]),
        make_record(
            "src/utils.rs",
            &["std::collections::HashSet", "std::path::Path"],
        ),
        make_record("src/errors.rs", &["std::fmt"]),
        make_record(
            "src/config.rs",
            &["std::collections::BTreeMap", "std::path::PathBuf"],
        ),
    ];

    let dg = build_dependency_graph(&records);

    assert_eq!(dg.node_count(), 5, "simple-rust fixture must have 5 nodes");
    assert_eq!(
        dg.edge_count(),
        0,
        "simple-rust fixture must have 0 edges (all imports external)"
    );
}

// ── hand-built graphs with known metrics ────────────────────────────────────

#[test]
fn empty_graph_metrics() {
    let dg = build_dependency_graph(&[]);
    let m = compute_metrics(&dg);
    assert_eq!(m.node_count, 0);
    assert_eq!(m.edge_count, 0);
    assert_eq!(m.density, 0.0);
    assert_eq!(m.cycle_count, 0);
    assert_eq!(m.component_count, 0);
}

#[test]
fn single_node_no_self_loop() {
    let records = vec![make_record("src/lib.rs", &[])];
    let dg = build_dependency_graph(&records);
    let m = compute_metrics(&dg);
    assert_eq!(m.node_count, 1);
    assert_eq!(m.edge_count, 0);
    assert_eq!(m.density, 0.0);
    assert_eq!(m.cycle_count, 0);
    assert_eq!(m.component_count, 1);
}

#[test]
fn two_nodes_one_edge_density() {
    // A → B: density = 1 / (2*1) = 0.5
    let records = vec![
        make_record("src/lib.rs", &["crate::models"]),
        make_record("src/models.rs", &[]),
    ];
    let dg = build_dependency_graph(&records);
    let m = compute_metrics(&dg);
    assert_eq!(m.node_count, 2);
    assert_eq!(m.edge_count, 1);
    assert!((m.density - 0.5).abs() < 1e-9, "density = {}", m.density);
    assert_eq!(m.cycle_count, 0);
    assert_eq!(m.component_count, 1);
}

#[test]
fn two_nodes_mutual_edge_one_cycle() {
    // A → B and B → A: one back-edge detected
    let records = vec![
        make_record("src/lib.rs", &["crate::models"]),
        make_record("src/models.rs", &["crate::lib"]),
    ];
    let dg = build_dependency_graph(&records);
    let m = compute_metrics(&dg);
    assert_eq!(m.node_count, 2);
    assert_eq!(m.edge_count, 2);
    assert_eq!(m.cycle_count, 1, "one back-edge = one cycle");
    assert_eq!(m.component_count, 1);
}

#[test]
fn three_disconnected_nodes_three_components() {
    let records = vec![
        make_record("src/a.rs", &[]),
        make_record("src/b.rs", &[]),
        make_record("src/c.rs", &[]),
    ];
    let dg = build_dependency_graph(&records);
    let m = compute_metrics(&dg);
    assert_eq!(m.node_count, 3);
    assert_eq!(m.edge_count, 0);
    assert_eq!(m.component_count, 3);
}

#[test]
fn chain_a_b_c_one_component_no_cycles() {
    // A → B → C: no cycles, one component
    let records = vec![
        make_record("src/a.rs", &["crate::b"]),
        make_record("src/b.rs", &["crate::c"]),
        make_record("src/c.rs", &[]),
    ];
    let dg = build_dependency_graph(&records);
    let m = compute_metrics(&dg);
    assert_eq!(m.node_count, 3);
    assert_eq!(m.edge_count, 2);
    assert_eq!(m.cycle_count, 0);
    assert_eq!(m.component_count, 1);
}

#[test]
fn self_loop_not_counted_as_cycle() {
    // Self-import strings don't resolve to the same node via stem matching
    // (relative import `crate::a` from `src/a.rs` would match itself, but
    // the builder skips self-loops). Verify with a distinct file that imports itself.
    let records = vec![make_record("src/a.rs", &["crate::a"])];
    // `crate::a` resolves to the single `a.rs` node — a self-loop.
    let dg = build_dependency_graph(&records);
    // Self-loops are dropped by the builder.
    assert_eq!(dg.edge_count(), 0, "self-loop must be dropped");
    let m = compute_metrics(&dg);
    assert_eq!(m.cycle_count, 0, "self-loop must not count as a cycle");
}

#[test]
fn top_hubs_sorted_by_out_degree() {
    // A imports from B and C; B imports from C; C imports nothing.
    // Out-degrees: A=2, B=1, C=0.
    let records = vec![
        make_record("src/a.rs", &["crate::b", "crate::c"]),
        make_record("src/b.rs", &["crate::c"]),
        make_record("src/c.rs", &[]),
    ];
    let dg = build_dependency_graph(&records);
    let m = compute_metrics(&dg);
    assert_eq!(m.top_hubs[0].1, 2, "highest hub has out-degree 2");
    assert_eq!(m.top_hubs[1].1, 1, "second hub has out-degree 1");
}

#[test]
fn duplicate_import_does_not_add_duplicate_edge() {
    let records = vec![
        make_record("src/a.rs", &["crate::b", "crate::b"]),
        make_record("src/b.rs", &[]),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        "duplicate import must produce only one edge"
    );
}

#[test]
fn triangle_cycle_count_one() {
    // A→B, B→C, C→A: one cycle
    let records = vec![
        make_record("src/a.rs", &["crate::b"]),
        make_record("src/b.rs", &["crate::c"]),
        make_record("src/c.rs", &["crate::a"]),
    ];
    let dg = build_dependency_graph(&records);
    let m = compute_metrics(&dg);
    assert_eq!(m.cycle_count, 1);
}

#[test]
fn density_complete_graph_three_nodes() {
    // 3-node complete directed graph (no self-loops): 3*2 = 6 possible edges.
    let records = vec![
        make_record("src/a.rs", &["crate::b", "crate::c"]),
        make_record("src/b.rs", &["crate::a", "crate::c"]),
        make_record("src/c.rs", &["crate::a", "crate::b"]),
    ];
    let dg = build_dependency_graph(&records);
    let m = compute_metrics(&dg);
    assert_eq!(m.node_count, 3);
    assert_eq!(m.edge_count, 6);
    assert!((m.density - 1.0).abs() < 1e-9);
}
