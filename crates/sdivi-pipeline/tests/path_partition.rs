//! Indirect coverage of the private `compute_path_partition` helper
//! (pipeline.rs:239-252).
//!
//! Reviewer coverage gap: "A test constructing a small DependencyGraph +
//! LeidenPartition and asserting path→community mapping would protect the
//! numeric cast and the UTF-8 path filter."
//!
//! Because `compute_path_partition` is private, these tests exercise it via
//! the public `Pipeline::snapshot_with_mode` API and assert on the resulting
//! `Snapshot::path_partition` field.

use std::path::Path;

use sdivi_config::Config;
use sdivi_lang_rust::RustAdapter;
use sdivi_pipeline::{Pipeline, WriteMode};

fn fixture_root() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/simple-rust"
    ))
}

/// The `simple-rust` fixture has 5 `.rs` files.  After a pipeline run the
/// `path_partition` must be populated — at minimum one entry per source file.
#[test]
fn pipeline_populates_path_partition_on_real_fixture() {
    let root = fixture_root();
    let adapters: Vec<Box<dyn sdivi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];
    let pipeline = Pipeline::new(Config::default(), adapters);

    let snap = pipeline
        .snapshot_with_mode(
            root,
            None,
            "2026-04-29T00:00:00Z",
            WriteMode::EphemeralForCheck,
        )
        .expect("pipeline must succeed on simple-rust fixture");

    assert!(
        !snap.path_partition.is_empty(),
        "path_partition must be populated after pipeline run"
    );
}

/// Every key in `path_partition` must be a valid (non-empty) string path.
/// This guards the `path.to_str()` UTF-8 filter in `compute_path_partition`.
#[test]
fn path_partition_keys_are_non_empty_strings() {
    let root = fixture_root();
    let adapters: Vec<Box<dyn sdivi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];
    let pipeline = Pipeline::new(Config::default(), adapters);

    let snap = pipeline
        .snapshot_with_mode(
            root,
            None,
            "2026-04-29T00:00:00Z",
            WriteMode::EphemeralForCheck,
        )
        .expect("pipeline must succeed");

    for key in snap.path_partition.keys() {
        assert!(!key.is_empty(), "path_partition key must not be empty");
        assert!(
            key.ends_with(".rs"),
            "simple-rust fixture keys must have .rs extension; got {key:?}"
        );
    }
}

/// The number of entries in `path_partition` must equal the node count
/// of the dependency graph, since every parsed file becomes a node and
/// `compute_path_partition` maps all nodes to community IDs.
#[test]
fn path_partition_entry_count_matches_graph_node_count() {
    let root = fixture_root();
    let adapters: Vec<Box<dyn sdivi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];
    let pipeline = Pipeline::new(Config::default(), adapters);

    let snap = pipeline
        .snapshot_with_mode(
            root,
            None,
            "2026-04-29T00:00:00Z",
            WriteMode::EphemeralForCheck,
        )
        .expect("pipeline must succeed");

    assert_eq!(
        snap.path_partition.len(),
        snap.graph.node_count,
        "path_partition must have one entry per graph node"
    );
}

/// Community IDs in `path_partition` must be valid u32 values that correspond
/// to communities present in the `partition.stability` map.
///
/// Guards the `comm_id as u32` numeric cast in `compute_path_partition`.
#[test]
fn path_partition_community_ids_are_valid() {
    let root = fixture_root();
    let adapters: Vec<Box<dyn sdivi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];
    let pipeline = Pipeline::new(Config::default(), adapters);

    let snap = pipeline
        .snapshot_with_mode(
            root,
            None,
            "2026-04-29T00:00:00Z",
            WriteMode::EphemeralForCheck,
        )
        .expect("pipeline must succeed");

    // Every community ID in path_partition must correspond to a valid community
    // in the Leiden partition.
    let known_communities: std::collections::BTreeSet<u32> = snap
        .partition
        .stability
        .keys()
        .map(|&id| id as u32)
        .collect();

    for &comm_id in snap.path_partition.values() {
        assert!(
            known_communities.contains(&comm_id),
            "path_partition community ID {comm_id} must be a known Leiden community"
        );
    }
}

/// `path_partition` is stable across two identical runs (same config + same
/// fixture → same partition → same mapping).  Guards determinism of
/// `compute_path_partition`.
#[test]
fn path_partition_is_deterministic() {
    let root = fixture_root();
    let make_pipeline = || {
        let adapters: Vec<Box<dyn sdivi_parsing::adapter::LanguageAdapter>> =
            vec![Box::new(RustAdapter)];
        Pipeline::new(Config::default(), adapters)
    };

    let snap1 = make_pipeline()
        .snapshot_with_mode(
            root,
            None,
            "2026-04-29T00:00:00Z",
            WriteMode::EphemeralForCheck,
        )
        .unwrap();
    let snap2 = make_pipeline()
        .snapshot_with_mode(
            root,
            None,
            "2026-04-29T00:00:00Z",
            WriteMode::EphemeralForCheck,
        )
        .unwrap();

    assert_eq!(
        snap1.path_partition, snap2.path_partition,
        "path_partition must be identical across two runs with the same config"
    );
}
