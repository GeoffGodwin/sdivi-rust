//! Boundary inference helpers — reads snapshot history, builds prior-partition
//! slices, and delegates to `sdivi_core::infer_boundaries`.

use std::path::Path;

use sdivi_snapshot::boundary_inference::{BoundaryInferenceResult, PriorPartition};

use crate::store::read_snapshots;

/// Reads the last `n` snapshots from `snapshot_dir` and converts each
/// snapshot's `path_partition` into a [`PriorPartition`] (oldest → newest).
///
/// Snapshots that have an empty `path_partition` (e.g., produced by an older
/// version of sdivi-rust or by the pure-compute path) are included as empty
/// partitions, which effectively lower the stability count for all communities.
///
/// Returns an empty `Vec` when the directory does not exist or contains fewer
/// than `n` snapshots (all available snapshots are returned in that case).
pub fn read_prior_partitions(
    snapshot_dir: &Path,
    n: usize,
) -> std::io::Result<Vec<PriorPartition>> {
    if n == 0 {
        return Ok(vec![]);
    }
    let mut snapshots = read_snapshots(snapshot_dir)?;
    if snapshots.len() > n {
        let skip = snapshots.len() - n;
        snapshots.drain(..skip);
    }
    let partitions = snapshots
        .into_iter()
        .map(|snap| PriorPartition {
            cluster_assignments: snap.path_partition,
        })
        .collect();
    Ok(partitions)
}

/// Infers boundary proposals from the stored snapshot history.
///
/// Reads `stability_threshold + 1` snapshots from `snapshot_dir`, assembles a
/// `Vec<PriorPartition>` (oldest → newest), and calls
/// [`sdivi_core::infer_boundaries`].
///
/// Returns an empty `BoundaryInferenceResult` when there are no snapshots or
/// when not enough history exists to satisfy `stability_threshold`.
pub fn infer_from_snapshots(
    snapshot_dir: &Path,
    stability_threshold: u32,
) -> std::io::Result<BoundaryInferenceResult> {
    let n = (stability_threshold as usize).saturating_add(1);
    let partitions = read_prior_partitions(snapshot_dir, n)?;
    Ok(sdivi_core::infer_boundaries(
        &partitions,
        stability_threshold,
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use tempfile::TempDir;

    use super::*;

    fn write_fake_snapshot(dir: &TempDir, name: &str, pp: BTreeMap<String, u32>) {
        use sdivi_detection::partition::LeidenPartition;
        use sdivi_graph::metrics::GraphMetrics;
        use sdivi_patterns::PatternCatalog;
        use sdivi_snapshot::snapshot::{assemble_snapshot, PatternMetricsResult};

        let mut snap = assemble_snapshot(
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
            "2026-04-29T00:00:00Z",
            None,
            None,
            0,
        );
        snap.path_partition = pp;
        let json = serde_json::to_string(&snap).unwrap();
        let snap_dir = dir.path().join(".sdivi").join("snapshots");
        std::fs::create_dir_all(&snap_dir).unwrap();
        std::fs::write(snap_dir.join(name), json).unwrap();
    }

    #[test]
    fn empty_dir_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let snap_dir = dir.path().join(".sdivi").join("snapshots");
        let result = infer_from_snapshots(&snap_dir, 2).unwrap();
        assert!(result.proposals.is_empty());
    }

    #[test]
    fn stable_community_is_proposed() {
        let dir = tempfile::tempdir().unwrap();
        let snap_dir = dir.path().join(".sdivi").join("snapshots");
        let pp: BTreeMap<String, u32> =
            [("a.rs".to_string(), 0u32), ("b.rs".to_string(), 0u32)].into();
        write_fake_snapshot(&dir, "snapshot_001.json", pp.clone());
        write_fake_snapshot(&dir, "snapshot_002.json", pp.clone());
        write_fake_snapshot(&dir, "snapshot_003.json", pp.clone());

        let result = infer_from_snapshots(&snap_dir, 2).unwrap();
        assert!(
            !result.proposals.is_empty(),
            "expected at least one proposal"
        );
    }

    #[test]
    fn read_prior_partitions_limits_to_n() {
        let dir = tempfile::tempdir().unwrap();
        let snap_dir = dir.path().join(".sdivi").join("snapshots");
        for i in 0..5u32 {
            let pp = [(format!("file_{i}.rs"), i)].into_iter().collect();
            write_fake_snapshot(&dir, &format!("snapshot_{i:03}.json"), pp);
        }
        let partitions = read_prior_partitions(&snap_dir, 3).unwrap();
        assert_eq!(partitions.len(), 3);
    }
}
