//! Property-based determinism test: same seed + same graph → same partition.
//!
//! `prop_test_leiden_seeded` runs Leiden 100 times on the same ring-of-cliques
//! graph with the same seed and asserts bit-identical results every time.

use proptest::prelude::*;
use sdi_detection::leiden::run_leiden;
use sdi_detection::partition::LeidenConfig;
use sdi_graph::dependency_graph::build_dependency_graph;
use sdi_parsing::feature_record::FeatureRecord;
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

/// Builds a small ring-of-cliques fixture: 3 cliques of 4 nodes, each clique
/// connected to the next by a single edge.  Produces a stable community
/// structure that Leiden reliably partitions into 3 communities.
fn ring_of_cliques_records() -> Vec<FeatureRecord> {
    vec![
        // Clique 0: a0, a1, a2, a3 fully connected
        make_record("src/a0.rs", &["crate::a1", "crate::a2", "crate::a3"]),
        make_record("src/a1.rs", &["crate::a0", "crate::a2", "crate::a3"]),
        make_record("src/a2.rs", &["crate::a0", "crate::a1", "crate::a3"]),
        make_record("src/a3.rs", &["crate::a0", "crate::a1", "crate::a2", "crate::b0"]),
        // Clique 1: b0, b1, b2, b3 fully connected
        make_record("src/b0.rs", &["crate::b1", "crate::b2", "crate::b3", "crate::a3"]),
        make_record("src/b1.rs", &["crate::b0", "crate::b2", "crate::b3"]),
        make_record("src/b2.rs", &["crate::b0", "crate::b1", "crate::b3"]),
        make_record("src/b3.rs", &["crate::b0", "crate::b1", "crate::b2", "crate::c0"]),
        // Clique 2: c0, c1, c2, c3 fully connected
        make_record("src/c0.rs", &["crate::c1", "crate::c2", "crate::c3", "crate::b3"]),
        make_record("src/c1.rs", &["crate::c0", "crate::c2", "crate::c3"]),
        make_record("src/c2.rs", &["crate::c0", "crate::c1", "crate::c3"]),
        make_record("src/c3.rs", &["crate::c0", "crate::c1", "crate::c2"]),
    ]
}

#[test]
fn prop_test_leiden_seeded() {
    let records = ring_of_cliques_records();
    let dg = build_dependency_graph(&records);
    let cfg = LeidenConfig { seed: 42, ..LeidenConfig::default() };

    // Run 100 times; every result must be bit-identical.
    let reference = run_leiden(&dg, &cfg, None);
    let ref_json = reference.to_json().unwrap();

    for run in 1..=100 {
        let result = run_leiden(&dg, &cfg, None);
        let json = result.to_json().unwrap();
        assert_eq!(
            json, ref_json,
            "Leiden run {run} produced different JSON for seed=42"
        );
    }
}

#[test]
fn different_seeds_may_differ() {
    let records = ring_of_cliques_records();
    let dg = build_dependency_graph(&records);

    let p42 = run_leiden(&dg, &LeidenConfig { seed: 42, ..LeidenConfig::default() }, None);
    let p99 = run_leiden(&dg, &LeidenConfig { seed: 99, ..LeidenConfig::default() }, None);

    // Both should produce valid partitions regardless of whether they differ.
    assert!(p42.community_count() > 0);
    assert!(p99.community_count() > 0);
}

proptest! {
    /// Property: for any seed in [0, 2^32), the same seed always produces the
    /// same partition on the ring-of-cliques fixture.
    #[test]
    fn prop_any_seed_deterministic(seed in 0u64..=u32::MAX as u64) {
        let records = ring_of_cliques_records();
        let dg = build_dependency_graph(&records);
        let cfg = LeidenConfig { seed, ..LeidenConfig::default() };

        let first = run_leiden(&dg, &cfg, None);
        let second = run_leiden(&dg, &cfg, None);

        prop_assert_eq!(
            first.to_json().unwrap(),
            second.to_json().unwrap(),
            "non-determinism for seed={}",
            seed
        );
    }
}

#[test]
fn disconnected_graph_each_component_partitioned() {
    // Two isolated cliques of 4 nodes — Leiden should produce ≥ 2 communities.
    let records = vec![
        make_record("src/a0.rs", &["crate::a1", "crate::a2"]),
        make_record("src/a1.rs", &["crate::a0", "crate::a2"]),
        make_record("src/a2.rs", &["crate::a0", "crate::a1"]),
        make_record("src/b0.rs", &["crate::b1", "crate::b2"]),
        make_record("src/b1.rs", &["crate::b0", "crate::b2"]),
        make_record("src/b2.rs", &["crate::b0", "crate::b1"]),
    ];
    let dg = build_dependency_graph(&records);
    let partition = run_leiden(&dg, &LeidenConfig::default(), None);
    assert!(
        partition.community_count() >= 2,
        "disconnected components must be in separate communities"
    );
}

#[test]
fn empty_graph_produces_empty_partition() {
    let dg = build_dependency_graph(&[]);
    let p = run_leiden(&dg, &LeidenConfig::default(), None);
    assert_eq!(p.community_count(), 0);
    assert_eq!(p.assignments.len(), 0);
}
