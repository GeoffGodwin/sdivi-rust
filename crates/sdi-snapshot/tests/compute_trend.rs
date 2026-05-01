use std::collections::BTreeMap;

use sdi_detection::partition::LeidenPartition;
use sdi_graph::metrics::GraphMetrics;
use sdi_patterns::PatternCatalog;
use sdi_snapshot::snapshot::{assemble_snapshot, PatternMetricsResult};
use sdi_snapshot::trend::compute_trend;

fn make_snap(density: f64, communities: usize) -> sdi_snapshot::snapshot::Snapshot {
    let mut stability = BTreeMap::new();
    for i in 0..communities {
        stability.insert(i, 1.0f64);
    }
    let graph = GraphMetrics {
        node_count: 4, edge_count: 1, density,
        cycle_count: 0, top_hubs: vec![], component_count: 1,
    };
    let partition = LeidenPartition {
        assignments: BTreeMap::new(), stability,
        modularity: 0.0, seed: 42,
    };
    assemble_snapshot(
        graph, partition, PatternCatalog::default(),
        PatternMetricsResult::default(), None, "2026-01-01T00:00:00Z", None, None,
    )
}

// ── Empty / small inputs ──────────────────────────────────────────────────────

#[test]
fn empty_slice_returns_zero_count() {
    let r = compute_trend(&[], None);
    assert_eq!(r.snapshot_count, 0);
    assert!(r.coupling_slope.is_none());
    assert!(r.pattern_entropy_slope.is_none());
    assert!(r.community_count_slope.is_none());
    assert!(r.convention_drift_slope.is_none());
}

#[test]
fn single_snapshot_no_slopes() {
    let snaps = vec![make_snap(0.5, 3)];
    let r = compute_trend(&snaps, None);
    assert_eq!(r.snapshot_count, 1);
    assert!(r.coupling_slope.is_none());
}

// ── Correctness ───────────────────────────────────────────────────────────────

#[test]
fn two_snapshots_coupling_slope() {
    let snaps = vec![make_snap(0.1, 2), make_snap(0.3, 4)];
    let r = compute_trend(&snaps, None);
    assert_eq!(r.snapshot_count, 2);
    let slope = r.coupling_slope.unwrap();
    assert!((slope - 0.2).abs() < 1e-10, "expected slope 0.2, got {slope}");
}

#[test]
fn two_snapshots_community_count_slope() {
    let snaps = vec![make_snap(0.0, 2), make_snap(0.0, 5)];
    let r = compute_trend(&snaps, None);
    assert_eq!(r.community_count_slope, Some(3.0));
}

#[test]
fn flat_series_slope_is_zero() {
    let snaps: Vec<_> = (0..4).map(|_| make_snap(0.5, 3)).collect();
    let r = compute_trend(&snaps, None);
    assert!((r.coupling_slope.unwrap()).abs() < 1e-10);
    assert!((r.community_count_slope.unwrap()).abs() < 1e-10);
}

// ── last_n clamping ───────────────────────────────────────────────────────────

#[test]
fn last_n_none_uses_all() {
    let snaps: Vec<_> = (0..5).map(|i| make_snap(i as f64 * 0.1, i + 1)).collect();
    let r = compute_trend(&snaps, None);
    assert_eq!(r.snapshot_count, 5);
}

#[test]
fn last_n_larger_than_len_uses_all() {
    let snaps = vec![make_snap(0.1, 2), make_snap(0.2, 3)];
    let r = compute_trend(&snaps, Some(100));
    assert_eq!(r.snapshot_count, 2);
}

#[test]
fn last_n_selects_tail() {
    // First 2 snapshots have density 0.0; last 2 have density 0.1 and 0.3.
    let snaps = vec![
        make_snap(0.0, 1),
        make_snap(0.0, 1),
        make_snap(0.1, 2),
        make_snap(0.3, 4),
    ];
    let r = compute_trend(&snaps, Some(2));
    assert_eq!(r.snapshot_count, 2);
    let slope = r.coupling_slope.unwrap();
    assert!((slope - 0.2).abs() < 1e-10);
}

#[test]
fn last_n_zero_gives_empty_result() {
    let snaps = vec![make_snap(0.5, 3), make_snap(0.6, 4)];
    let r = compute_trend(&snaps, Some(0));
    assert_eq!(r.snapshot_count, 0);
    assert!(r.coupling_slope.is_none());
}
