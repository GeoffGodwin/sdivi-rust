use std::collections::BTreeMap;

use sdi_detection::partition::LeidenPartition;
use sdi_graph::metrics::GraphMetrics;
use sdi_patterns::PatternCatalog;
use sdi_snapshot::build_snapshot;
use sdi_snapshot::enforce_retention;
use sdi_snapshot::write_snapshot;

fn snap_at(ts: &str) -> sdi_snapshot::Snapshot {
    build_snapshot(
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
        None,
        ts,
        None,
    )
}

fn count_snapshots(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir)
        .unwrap()
        .filter(|e| {
            let name = e.as_ref().unwrap().file_name().into_string().unwrap();
            name.starts_with("snapshot_") && name.ends_with(".json")
        })
        .count()
}

/// Writing max+1 snapshots then enforcing retention leaves exactly max files.
#[test]
fn write_n_plus_one_keeps_n() {
    let dir = tempfile::tempdir().unwrap();
    let max = 3u32;

    for i in 1..=(max + 1) {
        let ts = format!("2026040{i}T000000Z");
        write_snapshot(&snap_at(&ts), dir.path()).unwrap();
    }

    enforce_retention(dir.path(), max).unwrap();
    assert_eq!(count_snapshots(dir.path()), max as usize);
}

/// After writing four snapshots and enforcing max=3, the oldest is deleted.
#[test]
fn oldest_is_deleted_first() {
    let dir = tempfile::tempdir().unwrap();

    let timestamps = [
        "2026-01-01T00:00:00Z",
        "2026-01-02T00:00:00Z",
        "2026-01-03T00:00:00Z",
        "2026-01-04T00:00:00Z",
    ];

    for ts in &timestamps {
        write_snapshot(&snap_at(ts), dir.path()).unwrap();
    }

    enforce_retention(dir.path(), 3).unwrap();

    let names: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();

    // The oldest snapshot (20260101) must be gone.
    let has_oldest = names.iter().any(|n| n.contains("20260101"));
    assert!(!has_oldest, "oldest snapshot (20260101) should have been deleted");

    // Three should remain.
    assert_eq!(names.len(), 3);
}

/// enforce_retention(dir, 0) keeps all files (unlimited).
#[test]
fn retention_zero_keeps_all() {
    let dir = tempfile::tempdir().unwrap();

    for i in 1..=10u32 {
        let ts = format!("2026040{i}T000000Z");
        write_snapshot(&snap_at(&ts), dir.path()).unwrap();
    }

    enforce_retention(dir.path(), 0).unwrap();
    assert_eq!(count_snapshots(dir.path()), 10);
}

/// A single snapshot is never deleted regardless of the max setting.
#[test]
fn single_file_no_delete() {
    let dir = tempfile::tempdir().unwrap();
    write_snapshot(&snap_at("2026-01-01T00:00:00Z"), dir.path()).unwrap();

    enforce_retention(dir.path(), 5).unwrap();
    assert_eq!(count_snapshots(dir.path()), 1);
}
