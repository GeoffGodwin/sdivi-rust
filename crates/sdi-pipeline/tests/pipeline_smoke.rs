//! Smoke tests for `sdi_pipeline::Pipeline`.
//!
//! These tests verify that the five-stage orchestration pipeline can construct,
//! run `snapshot`, and produce a valid `Snapshot` with expected shape.

use std::path::Path;

use sdi_config::Config;
use sdi_lang_rust::RustAdapter;
use sdi_pipeline::Pipeline;
use sdi_snapshot::snapshot::SNAPSHOT_VERSION;

fn fixture_root() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/simple-rust"
    ))
}

#[test]
fn pipeline_new_is_cheap() {
    // Construction must be O(1): no I/O, no parsing.
    let _ = Pipeline::new(Config::default(), vec![]);
}

#[test]
fn snapshot_on_simple_rust_fixture() {
    // M16 changed commit=Some(ref) to trigger real git rev-parse resolution,
    // so passing a bare label no longer works. Use None (no-commit path) to
    // verify the pipeline works on the simple-rust fixture.
    let root = fixture_root();
    let adapters: Vec<Box<dyn sdi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];
    let pipeline = Pipeline::new(Config::default(), adapters);

    let snap = pipeline
        .snapshot(root, None, "2026-04-29T00:00:00Z")
        .expect("snapshot must succeed on simple-rust fixture");

    assert_eq!(snap.snapshot_version, SNAPSHOT_VERSION, "must emit schema version 1.0");
    // No commit ref supplied → commit field is absent.
    assert!(snap.commit.is_none(), "commit must be None when no ref is supplied");
    assert_eq!(snap.timestamp, "2026-04-29T00:00:00Z");
    // Five .rs files in simple-rust → at least one node.
    assert!(snap.graph.node_count > 0, "graph must have nodes");
}

#[test]
fn delta_of_same_snapshot_is_all_zero() {
    let root = fixture_root();
    let adapters: Vec<Box<dyn sdi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];
    let pipeline = Pipeline::new(Config::default(), adapters);
    let snap = pipeline.snapshot(root, None, "2026-04-29T00:00:00Z").unwrap();
    let summary = Pipeline::delta(Some(&snap), &snap);
    // Same-snapshot delta should have zero coupling_delta.
    assert_eq!(summary.coupling_delta, Some(0.0));
    assert_eq!(summary.community_count_delta, Some(0));
}
