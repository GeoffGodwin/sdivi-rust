use sdi_snapshot::PatternMetricsResult;
use std::collections::BTreeMap;

use sdi_detection::partition::LeidenPartition;
use sdi_graph::metrics::GraphMetrics;
use sdi_patterns::PatternCatalog;
use sdi_snapshot::assemble_snapshot;
use sdi_snapshot::write_snapshot;

fn empty_snap(ts: &str) -> sdi_snapshot::Snapshot {
    assemble_snapshot(
        GraphMetrics {
            node_count: 0,
            edge_count: 0,
            density: 0.0,
            cycle_count: 0,
            top_hubs: vec![],
            component_count: 0,
        },
        LeidenPartition {
            assignments: BTreeMap::new(),
            stability: BTreeMap::new(),
            modularity: 0.0,
            seed: 42,
        },
        PatternCatalog::default(),
        PatternMetricsResult::default(),
        None,
        ts,
        None,
        None,
    )
}

/// Writing one snapshot produces exactly one `.json` file named `snapshot_*`.
#[test]
fn write_creates_json_file() {
    let dir = tempfile::tempdir().unwrap();
    let snap = empty_snap("2026-04-29T00:00:00Z");
    write_snapshot(&snap, dir.path()).unwrap();

    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();

    assert_eq!(entries.len(), 1, "expected exactly one file, got: {entries:?}");
    assert!(
        entries[0].starts_with("snapshot_"),
        "file should start with 'snapshot_', got: {}",
        entries[0]
    );
    assert!(
        entries[0].ends_with(".json"),
        "file should end with '.json', got: {}",
        entries[0]
    );
}

/// The written file deserializes as valid JSON with `snapshot_version = "1.0"`.
#[test]
fn written_file_is_valid_json() {
    let dir = tempfile::tempdir().unwrap();
    let snap = empty_snap("2026-04-29T00:00:00Z");
    let path = write_snapshot(&snap, dir.path()).unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content)
        .expect("written file must be valid JSON");

    assert_eq!(
        parsed["snapshot_version"].as_str().unwrap(),
        "1.0",
        "snapshot_version must be '1.0'"
    );
}

/// The returned path resides inside the target directory, not in `/tmp`.
#[test]
fn written_file_in_target_dir_not_tmp() {
    let dir = tempfile::tempdir().unwrap();
    let snap = empty_snap("2026-04-29T00:00:00Z");
    let path = write_snapshot(&snap, dir.path()).unwrap();

    assert!(
        path.starts_with(dir.path()),
        "returned path {path:?} must be inside target dir {:?}",
        dir.path()
    );
}

/// After a successful write, no temporary files (non-`.json`) are left in the dir.
#[test]
fn no_temp_files_left_after_write() {
    let dir = tempfile::tempdir().unwrap();
    let snap = empty_snap("2026-04-29T00:00:00Z");
    write_snapshot(&snap, dir.path()).unwrap();

    let non_json_count = std::fs::read_dir(dir.path())
        .unwrap()
        .filter(|e| {
            let name = e.as_ref().unwrap().file_name().into_string().unwrap();
            !name.ends_with(".json")
        })
        .count();

    assert_eq!(
        non_json_count, 0,
        "no non-json files should remain after a successful write"
    );
}
