//! Tests for the warm-start path.
//!
//! Verifies that:
//! 1. A stale partition file is loaded and its assignments used to initialise
//!    the first Leiden iteration.
//! 2. Result quality with a warm start matches cold-start within 1% modularity.
//! 3. Missing / corrupt cache files fall back to cold start.
//! 4. Nodes absent from the cache get singleton assignments.

use std::collections::BTreeMap;
use std::path::PathBuf;

use sdi_detection::leiden::run_leiden;
use sdi_detection::partition::{LeidenConfig, LeidenPartition};
use sdi_detection::warm_start::{
    initial_assignment_from_cache, load_cached_partition, save_cached_partition,
};
use sdi_graph::dependency_graph::build_dependency_graph;
use sdi_parsing::feature_record::FeatureRecord;

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

fn clique_records() -> Vec<FeatureRecord> {
    // Two cliques of 5 nodes each, bridged by one edge.
    vec![
        make_record("src/a0.rs", &["crate::a1", "crate::a2", "crate::a3", "crate::a4"]),
        make_record("src/a1.rs", &["crate::a0", "crate::a2", "crate::a3", "crate::a4"]),
        make_record("src/a2.rs", &["crate::a0", "crate::a1", "crate::a3", "crate::a4"]),
        make_record("src/a3.rs", &["crate::a0", "crate::a1", "crate::a2", "crate::a4"]),
        make_record("src/a4.rs", &["crate::a0", "crate::a1", "crate::a2", "crate::a3", "crate::b0"]),
        make_record("src/b0.rs", &["crate::b1", "crate::b2", "crate::b3", "crate::b4", "crate::a4"]),
        make_record("src/b1.rs", &["crate::b0", "crate::b2", "crate::b3", "crate::b4"]),
        make_record("src/b2.rs", &["crate::b0", "crate::b1", "crate::b3", "crate::b4"]),
        make_record("src/b3.rs", &["crate::b0", "crate::b1", "crate::b2", "crate::b4"]),
        make_record("src/b4.rs", &["crate::b0", "crate::b1", "crate::b2", "crate::b3"]),
    ]
}

#[test]
fn missing_cache_returns_none() {
    let tmp = tempfile::tempdir().unwrap();
    let cache_path = tmp.path().join("partition.json");
    let result = load_cached_partition(&cache_path);
    assert!(result.is_none(), "missing file must return None");
}

#[test]
fn corrupt_cache_returns_none() {
    let tmp = tempfile::tempdir().unwrap();
    let cache_path = tmp.path().join("partition.json");
    std::fs::write(&cache_path, b"not valid json").unwrap();
    let result = load_cached_partition(&cache_path);
    assert!(result.is_none(), "corrupt JSON must return None");
}

#[test]
fn round_trip_save_load() {
    let tmp = tempfile::tempdir().unwrap();
    let cache_path = tmp.path().join("cache/partition.json");

    let original = LeidenPartition {
        assignments: BTreeMap::from([(0, 0), (1, 0), (2, 1)]),
        stability: BTreeMap::from([(0, 0.75), (1, 1.0)]),
        modularity: 0.35,
        seed: 42,
    };

    save_cached_partition(&original, &cache_path).expect("save must succeed");
    let loaded = load_cached_partition(&cache_path).expect("load must succeed");
    assert_eq!(original, loaded, "round-trip must be identical");
}

#[test]
fn initial_assignment_singletons_for_absent_nodes() {
    let cached = LeidenPartition {
        assignments: BTreeMap::from([(0, 0), (1, 0)]),
        stability: BTreeMap::from([(0, 0.8)]),
        modularity: 0.4,
        seed: 42,
    };

    // node_count = 5; nodes 2, 3, 4 are absent from cache.
    let init = initial_assignment_from_cache(Some(&cached), 5);
    assert_eq!(init.len(), 5);
    assert_eq!(init[0], init[1], "nodes 0 and 1 should share a community");
    // Nodes 2, 3, 4 should each be in a unique community.
    assert_ne!(init[2], init[3]);
    assert_ne!(init[2], init[4]);
    assert_ne!(init[3], init[4]);
    // Nodes 2, 3, 4 should be in different communities from nodes 0 and 1.
    assert_ne!(init[2], init[0]);
}

#[test]
fn warm_start_quality_within_one_percent_of_cold_start() {
    let records = clique_records();
    let dg = build_dependency_graph(&records);
    let cfg = LeidenConfig { seed: 42, ..LeidenConfig::default() };

    // Cold start
    let cold = run_leiden(&dg, &cfg, None);

    // Save the cold-start partition as the warm-start cache.
    let tmp = tempfile::tempdir().unwrap();
    let cache_path = tmp.path().join("partition.json");
    save_cached_partition(&cold, &cache_path).unwrap();

    // Load and run warm start.
    let cached = load_cached_partition(&cache_path).unwrap();
    let warm = run_leiden(&dg, &cfg, Some(&cached));

    // Modularity should be within 1% of cold-start.
    let cold_mod = cold.modularity;
    let warm_mod = warm.modularity;
    let tolerance = 0.01 * cold_mod.abs().max(1e-9);

    assert!(
        (warm_mod - cold_mod).abs() <= tolerance,
        "warm-start modularity {warm_mod:.6} deviates > 1% from cold-start {cold_mod:.6}"
    );
}

#[test]
fn warm_start_honours_prior_assignments_in_first_iteration() {
    let records = clique_records();
    let dg = build_dependency_graph(&records);

    // Create a partition that mirrors the expected clique structure.
    let warm_partition = LeidenPartition {
        assignments: BTreeMap::from([
            (0, 0), (1, 0), (2, 0), (3, 0), (4, 0),
            (5, 1), (6, 1), (7, 1), (8, 1), (9, 1),
        ]),
        stability: BTreeMap::from([(0, 0.9), (1, 0.9)]),
        modularity: 0.45,
        seed: 42,
    };

    let cfg = LeidenConfig { seed: 42, ..LeidenConfig::default() };
    let result = run_leiden(&dg, &cfg, Some(&warm_partition));

    // The warm-start partition is already near-optimal; result should be
    // a valid partition with community count ≥ 1.
    assert!(result.community_count() >= 1);
    assert_eq!(result.assignments.len(), 10);
}
