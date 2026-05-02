//! Cross-check suite against leidenalg reference values.
//!
//! GATED: only compiled and run when `--features verify-leiden` is passed.
//! Requires Python ≥ 3.9 and `pip install leidenalg igraph`.
//!
//! Tests load adjacency lists and reference values from
//! `tests/fixtures/leiden-graphs/{small,medium,large}/` and assert that the
//! native Leiden implementation produces:
//!   - Modularity within 1% of the reference value
//!   - Community count within ±10% of the reference count

#![cfg(feature = "verify-leiden")]

use sdivi_detection::leiden::run_leiden;
use sdivi_detection::partition::LeidenConfig;
use sdivi_graph::dependency_graph::build_dependency_graph;
use sdivi_parsing::feature_record::FeatureRecord;
use std::path::{Path, PathBuf};

/// Metadata file format from `tools/generate-leiden-fixtures.py`.
#[derive(serde::Deserialize, Debug)]
struct FixtureMetadata {
    node_count: usize,
    #[allow(dead_code)]
    edge_count: usize,
    reference_modularity: f64,
    reference_community_count: usize,
}

/// Edge list file format: list of `[src, dst]` pairs.
type EdgeList = Vec<[usize; 2]>;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/leiden-graphs")
        .join(name)
}

fn load_fixture(name: &str) -> (Vec<FeatureRecord>, FixtureMetadata) {
    let dir = fixture_path(name);

    let meta_json = std::fs::read_to_string(dir.join("metadata.json"))
        .unwrap_or_else(|e| panic!("cannot read {name}/metadata.json: {e}"));
    let meta: FixtureMetadata = serde_json::from_str(&meta_json)
        .unwrap_or_else(|e| panic!("cannot parse {name}/metadata.json: {e}"));

    let edges_json = std::fs::read_to_string(dir.join("adjacency.json"))
        .unwrap_or_else(|e| panic!("cannot read {name}/adjacency.json: {e}"));
    let edges: EdgeList = serde_json::from_str(&edges_json)
        .unwrap_or_else(|e| panic!("cannot parse {name}/adjacency.json: {e}"));

    // Build synthetic FeatureRecords: one per node, imports from edges.
    let mut import_map: Vec<Vec<String>> = vec![vec![]; meta.node_count];
    for [src, dst] in &edges {
        import_map[*src].push(format!("crate::n{dst}"));
    }

    let records: Vec<FeatureRecord> = (0..meta.node_count)
        .map(|i| FeatureRecord {
            path: PathBuf::from(format!("src/n{i}.rs")),
            language: "rust".to_string(),
            imports: import_map[i].clone(),
            exports: vec![],
            signatures: vec![],
            pattern_hints: vec![],
        })
        .collect();

    (records, meta)
}

fn check_fixture(name: &str) {
    let (records, meta) = load_fixture(name);
    let dg = build_dependency_graph(&records);
    let cfg = LeidenConfig {
        seed: 42,
        ..LeidenConfig::default()
    };
    let partition = run_leiden(&dg, &cfg, None);

    let ref_mod = meta.reference_modularity;
    let our_mod = partition.modularity;

    // Modularity within 1% of reference.
    let tolerance = 0.01 * ref_mod.abs().max(1e-9);
    assert!(
        (our_mod - ref_mod).abs() <= tolerance,
        "[{name}] modularity {our_mod:.6} not within 1% of reference {ref_mod:.6}"
    );

    // Community count within ±10%.
    let ref_count = meta.reference_community_count;
    let our_count = partition.community_count();
    let count_tol = (ref_count as f64 * 0.10).ceil() as usize;
    assert!(
        our_count.abs_diff(ref_count) <= count_tol.max(1),
        "[{name}] community count {our_count} not within ±10% of reference {ref_count}"
    );

    // No community larger than 50% of node count.
    let max_size = partition.largest_community_size();
    assert!(
        max_size <= meta.node_count / 2 + 1,
        "[{name}] largest community {max_size} exceeds 50% of {}",
        meta.node_count
    );
}

#[test]
fn small_fixture_quality() {
    check_fixture("small");
}

#[test]
fn medium_fixture_quality() {
    check_fixture("medium");
}

#[test]
fn large_fixture_quality() {
    check_fixture("large");
}
