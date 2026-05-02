//! Smoke tests for `sdivi_pipeline::Pipeline`.
//!
//! These tests verify that the five-stage orchestration pipeline can construct,
//! run `snapshot`, and produce a valid `Snapshot` with expected shape.

use std::path::{Path, PathBuf};

use sdivi_config::Config;
use sdivi_lang_rust::RustAdapter;
use sdivi_pipeline::Pipeline;
use sdivi_snapshot::snapshot::SNAPSHOT_VERSION;

fn fixture_root() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/simple-rust"
    ))
}

/// Copies the read-only fixture into a fresh tempdir and returns the tempdir
/// + the copied repo root. Tests must not write directly into the fixture
/// — on Windows, parallel tests writing identical filenames into the
/// fixture's `.sdivi/snapshots/` race on file creation and one panics with
/// `Access is denied` (sharing violation). Per-test tempdirs sidestep this.
fn isolated_fixture() -> (tempfile::TempDir, PathBuf) {
    let src = fixture_root();
    let tmp = tempfile::TempDir::new().expect("temp fixture dir");
    let dst = tmp.path().join("repo");
    copy_dir_recursive(src, &dst).expect("copy fixture into tempdir");
    (tmp, dst)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        let name = entry.file_name();
        let from = entry.path();
        let to = dst.join(&name);
        if ft.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if ft.is_file() {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
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
    let (_tmp, root) = isolated_fixture();
    let adapters: Vec<Box<dyn sdivi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];
    let pipeline = Pipeline::new(Config::default(), adapters);

    let snap = pipeline
        .snapshot(&root, None, "2026-04-29T00:00:00Z")
        .expect("snapshot must succeed on simple-rust fixture");

    assert_eq!(
        snap.snapshot_version, SNAPSHOT_VERSION,
        "must emit schema version 1.0"
    );
    // No commit ref supplied → commit field is absent.
    assert!(
        snap.commit.is_none(),
        "commit must be None when no ref is supplied"
    );
    assert_eq!(snap.timestamp, "2026-04-29T00:00:00Z");
    // Five .rs files in simple-rust → at least one node.
    assert!(snap.graph.node_count > 0, "graph must have nodes");
}

#[test]
fn delta_of_same_snapshot_is_all_zero() {
    let (_tmp, root) = isolated_fixture();
    let adapters: Vec<Box<dyn sdivi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];
    let pipeline = Pipeline::new(Config::default(), adapters);
    let snap = pipeline
        .snapshot(&root, None, "2026-04-29T00:00:00Z")
        .unwrap();
    let summary = Pipeline::delta(Some(&snap), &snap);
    // Same-snapshot delta should have zero coupling_delta.
    assert_eq!(summary.coupling_delta, Some(0.0));
    assert_eq!(summary.community_count_delta, Some(0));
}
