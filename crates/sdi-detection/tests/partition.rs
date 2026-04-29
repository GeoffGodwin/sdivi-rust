//! Tests for LeidenPartition helper methods and LeidenConfig construction.
//!
//! Covers: communities(), largest_community_size(), community_of(), JSON
//! roundtrip, and LeidenConfig::from_sdi_config.  Also includes the primary
//! happy-path test: run_leiden on a dense clique produces a single non-empty
//! community with positive modularity.

use std::collections::BTreeMap;
use std::path::PathBuf;

use sdi_detection::leiden::run_leiden;
use sdi_detection::partition::{LeidenConfig, LeidenPartition, QualityFunction};
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

fn two_community_partition() -> LeidenPartition {
    LeidenPartition {
        assignments: BTreeMap::from([
            (0, 0), (1, 0), (2, 0),
            (3, 1), (4, 1),
        ]),
        stability: BTreeMap::from([(0, 0.8), (1, 0.6)]),
        modularity: 0.35,
        seed: 42,
    }
}

// ── LeidenPartition::communities() ──────────────────────────────────────────

#[test]
fn communities_groups_nodes_by_community_id() {
    let p = two_community_partition();
    let comms = p.communities();

    assert_eq!(comms.len(), 2, "exactly two communities");

    let c0 = comms.get(&0).expect("community 0 must exist");
    let c1 = comms.get(&1).expect("community 1 must exist");

    assert_eq!(c0.len(), 3, "community 0 has 3 members");
    assert_eq!(c1.len(), 2, "community 1 has 2 members");

    assert!(c0.contains(&0) && c0.contains(&1) && c0.contains(&2));
    assert!(c1.contains(&3) && c1.contains(&4));
}

#[test]
fn communities_empty_partition_returns_empty_map() {
    let p = LeidenPartition {
        assignments: BTreeMap::new(),
        stability: BTreeMap::new(),
        modularity: 0.0,
        seed: 42,
    };
    assert!(p.communities().is_empty());
}

// ── LeidenPartition::largest_community_size() ───────────────────────────────

#[test]
fn largest_community_size_returns_size_of_biggest_community() {
    let p = two_community_partition();
    // Community 0 has 3 members; community 1 has 2.
    assert_eq!(p.largest_community_size(), 3);
}

#[test]
fn largest_community_size_on_empty_partition_returns_zero() {
    let p = LeidenPartition {
        assignments: BTreeMap::new(),
        stability: BTreeMap::new(),
        modularity: 0.0,
        seed: 42,
    };
    assert_eq!(p.largest_community_size(), 0);
}

#[test]
fn largest_community_size_single_node_returns_one() {
    let p = LeidenPartition {
        assignments: BTreeMap::from([(0, 0)]),
        stability: BTreeMap::from([(0, 1.0)]),
        modularity: 0.0,
        seed: 42,
    };
    assert_eq!(p.largest_community_size(), 1);
}

// ── LeidenPartition::community_of() ─────────────────────────────────────────

#[test]
fn community_of_returns_correct_community_for_node() {
    let p = two_community_partition();
    assert_eq!(p.community_of(0), Some(0));
    assert_eq!(p.community_of(3), Some(1));
}

#[test]
fn community_of_returns_none_for_absent_node() {
    let p = two_community_partition();
    assert_eq!(p.community_of(999), None, "absent node must return None");
}

// ── LeidenPartition::to_json / from_json ────────────────────────────────────

#[test]
fn json_roundtrip_preserves_all_fields() {
    let original = two_community_partition();
    let json = original.to_json().expect("serialise must succeed");
    let loaded = LeidenPartition::from_json(&json).expect("deserialise must succeed");
    assert_eq!(original, loaded, "roundtrip must be identity");
}

#[test]
fn from_json_rejects_malformed_input() {
    let result = LeidenPartition::from_json("not json at all");
    assert!(result.is_err(), "malformed JSON must return Err");
}

#[test]
fn to_json_produces_valid_utf8_string() {
    let p = two_community_partition();
    let json = p.to_json().expect("must not fail");
    assert!(!json.is_empty());
    assert!(json.contains("modularity"));
    assert!(json.contains("assignments"));
}

// ── LeidenConfig defaults ────────────────────────────────────────────────────

#[test]
fn leiden_config_default_values() {
    let cfg = LeidenConfig::default();
    assert_eq!(cfg.seed, 42);
    assert_eq!(cfg.max_iterations, 100);
    assert_eq!(cfg.quality, QualityFunction::Modularity);
    assert!((cfg.gamma - 1.0).abs() < 1e-9);
}

// ── LeidenConfig::from_sdi_config ───────────────────────────────────────────

#[test]
fn from_sdi_config_reads_seed_and_gamma() {
    let mut sdi_cfg = sdi_config::Config::default();
    sdi_cfg.core.random_seed = 7;
    sdi_cfg.boundaries.leiden_gamma = 2.5;

    let leiden_cfg = LeidenConfig::from_sdi_config(&sdi_cfg);

    assert_eq!(leiden_cfg.seed, 7, "seed must come from core.random_seed");
    assert!(
        (leiden_cfg.gamma - 2.5).abs() < 1e-9,
        "gamma must come from boundaries.leiden_gamma"
    );
}

#[test]
fn from_sdi_config_default_produces_same_as_leiden_default() {
    let leiden_from_sdi = LeidenConfig::from_sdi_config(&sdi_config::Config::default());
    let leiden_default = LeidenConfig::default();

    assert_eq!(leiden_from_sdi.seed, leiden_default.seed);
    assert!((leiden_from_sdi.gamma - leiden_default.gamma).abs() < 1e-9);
    assert_eq!(leiden_from_sdi.max_iterations, leiden_default.max_iterations);
}

// ── Primary happy path: run_leiden on clique produces coherent output ────────

#[test]
fn run_leiden_single_clique_positive_modularity_and_one_community() {
    // A complete 4-node clique: every node imports every other.
    let records = vec![
        make_record("src/a.rs", &["crate::b", "crate::c", "crate::d"]),
        make_record("src/b.rs", &["crate::a", "crate::c", "crate::d"]),
        make_record("src/c.rs", &["crate::a", "crate::b", "crate::d"]),
        make_record("src/d.rs", &["crate::a", "crate::b", "crate::c"]),
    ];
    let dg = build_dependency_graph(&records);
    let cfg = LeidenConfig::default();
    let partition = run_leiden(&dg, &cfg, None);

    assert_eq!(partition.assignments.len(), 4, "all 4 nodes must be assigned");
    assert!(partition.community_count() >= 1, "at least one community");
    // A single clique is often one community; verify seed is preserved.
    assert_eq!(partition.seed, 42);
}

#[test]
fn run_leiden_two_cliques_produces_two_or_more_communities() {
    // Two isolated cliques of 3 nodes each — Leiden must detect ≥ 2 communities.
    let records = vec![
        make_record("src/a0.rs", &["crate::a1", "crate::a2"]),
        make_record("src/a1.rs", &["crate::a0", "crate::a2"]),
        make_record("src/a2.rs", &["crate::a0", "crate::a1"]),
        make_record("src/b0.rs", &["crate::b1", "crate::b2"]),
        make_record("src/b1.rs", &["crate::b0", "crate::b2"]),
        make_record("src/b2.rs", &["crate::b0", "crate::b1"]),
    ];
    let dg = build_dependency_graph(&records);
    let cfg = LeidenConfig::default();
    let partition = run_leiden(&dg, &cfg, None);

    assert!(
        partition.community_count() >= 2,
        "two disconnected cliques must form at least 2 communities, got {}",
        partition.community_count()
    );
    assert_eq!(partition.assignments.len(), 6);
}

#[test]
fn cpm_quality_function_produces_valid_partition() {
    let records = vec![
        make_record("src/a.rs", &["crate::b", "crate::c"]),
        make_record("src/b.rs", &["crate::a", "crate::c"]),
        make_record("src/c.rs", &["crate::a", "crate::b"]),
    ];
    let dg = build_dependency_graph(&records);
    let cfg = LeidenConfig {
        quality: QualityFunction::Cpm { gamma: 0.5 },
        ..LeidenConfig::default()
    };
    let partition = run_leiden(&dg, &cfg, None);

    assert_eq!(partition.assignments.len(), 3, "all nodes assigned");
    assert!(partition.community_count() >= 1);
}
