use sdivi_snapshot::PatternMetricsResult;
use std::collections::BTreeMap;
use std::io::ErrorKind;

use sdivi_detection::partition::LeidenPartition;
use sdivi_graph::metrics::GraphMetrics;
use sdivi_patterns::PatternCatalog;
use sdivi_snapshot::assemble_snapshot;
use sdivi_snapshot::write_snapshot;
use sdivi_snapshot::Snapshot;

fn sample_snap() -> Snapshot {
    assemble_snapshot(
        GraphMetrics {
            node_count: 3,
            edge_count: 2,
            density: 0.333,
            cycle_count: 0,
            top_hubs: vec![],
            component_count: 1,
        },
        LeidenPartition {
            assignments: BTreeMap::from([(0usize, 0usize), (1, 0), (2, 1)]),
            stability: BTreeMap::from([(0usize, 1.0f64), (1, 0.8)]),
            modularity: 0.25,
            seed: 42,
        },
        PatternCatalog::default(),
        PatternMetricsResult::default(),
        None,
        "2026-04-29T09:00:00Z",
        Some("deadbeefdeadbeefdeadbeef"),
        None,
    )
}

/// Writing a snapshot and loading it back produces a value equal to the original.
#[test]
fn load_round_trips_written_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let original = sample_snap();
    let path = write_snapshot(&original, dir.path()).unwrap();

    let loaded = Snapshot::load(&path).expect("load must succeed for a file we just wrote");

    assert_eq!(original, loaded, "loaded snapshot must equal the original");
}

/// Loading a file that does not exist returns Err with kind NotFound.
#[test]
fn load_missing_file_returns_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("does_not_exist.json");

    let err = Snapshot::load(&missing).expect_err("missing file must return Err");
    assert_eq!(
        err.kind(),
        ErrorKind::NotFound,
        "expected NotFound, got {:?}",
        err.kind()
    );
}

/// Loading a file containing invalid JSON returns Err with kind InvalidData.
#[test]
fn load_invalid_json_returns_invalid_data() {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad.json");
    std::fs::write(&bad, b"this is not valid json {{{{").unwrap();

    let err = Snapshot::load(&bad).expect_err("malformed JSON must return Err");
    assert_eq!(
        err.kind(),
        ErrorKind::InvalidData,
        "expected InvalidData, got {:?}",
        err.kind()
    );
}

/// Loading a valid JSON file that doesn't match the Snapshot schema also returns InvalidData.
#[test]
fn load_wrong_schema_returns_invalid_data() {
    let dir = tempfile::tempdir().unwrap();
    let wrong = dir.path().join("wrong_schema.json");
    std::fs::write(&wrong, b"{\"foo\": 42}").unwrap();

    let err = Snapshot::load(&wrong).expect_err("wrong-schema JSON must return Err");
    assert_eq!(
        err.kind(),
        ErrorKind::InvalidData,
        "expected InvalidData for schema mismatch, got {:?}",
        err.kind()
    );
}

/// The round-trip preserves the commit field exactly.
#[test]
fn load_preserves_commit_field() {
    let dir = tempfile::tempdir().unwrap();
    let original = sample_snap();
    let path = write_snapshot(&original, dir.path()).unwrap();
    let loaded = Snapshot::load(&path).unwrap();

    assert_eq!(
        loaded.commit.as_deref(),
        Some("deadbeefdeadbeefdeadbeef"),
        "commit field must survive the round-trip"
    );
}

/// The round-trip preserves the snapshot_version field as "1.0".
#[test]
fn load_preserves_snapshot_version() {
    let dir = tempfile::tempdir().unwrap();
    let original = sample_snap();
    let path = write_snapshot(&original, dir.path()).unwrap();
    let loaded = Snapshot::load(&path).unwrap();

    assert_eq!(
        loaded.snapshot_version, "1.0",
        "snapshot_version must be '1.0' after round-trip"
    );
}
