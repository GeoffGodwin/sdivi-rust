//! [`compute_trend`] — trend analysis over a sequence of snapshots.

use serde::{Deserialize, Serialize};

use crate::snapshot::Snapshot;

/// Per-dimension trend data over a window of consecutive snapshots.
///
/// All delta fields are the slope (change per snapshot interval) of a linear
/// regression over the window, or `None` when there are fewer than two snapshots
/// to compare.
///
/// # Examples
///
/// ```rust
/// use sdivi_snapshot::trend::compute_trend;
///
/// let result = compute_trend(&[], None);
/// assert_eq!(result.snapshot_count, 0);
/// assert!(result.pattern_entropy_slope.is_none());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrendResult {
    /// Number of snapshots in the analysis window.
    pub snapshot_count: usize,
    /// Slope of `pattern_entropy_delta` across the window (nats / snapshot).
    pub pattern_entropy_slope: Option<f64>,
    /// Slope of `convention_drift_delta` across the window.
    pub convention_drift_slope: Option<f64>,
    /// Slope of `coupling_delta` across the window.
    pub coupling_slope: Option<f64>,
    /// Slope of `community_count_delta` across the window.
    pub community_count_slope: Option<f64>,
}

/// Computes trend statistics over a slice of [`Snapshot`]s.
///
/// `last_n = None` uses all snapshots; `Some(n)` clamps to the `n` most recent
/// (an `n` larger than the slice length silently uses all).  Snapshots are
/// assumed to be ordered oldest→newest.
///
/// The slope is the mean of per-interval deltas (a simple first-difference
/// average rather than least-squares, which is sufficient for the CLI trend
/// command).
///
/// Returns a [`TrendResult`] with all slope fields `None` when the window has
/// fewer than two snapshots.
///
/// # Examples
///
/// ```rust
/// use sdivi_snapshot::trend::compute_trend;
///
/// // Empty slice → zero snapshots, all slopes None.
/// let r = compute_trend(&[], None);
/// assert_eq!(r.snapshot_count, 0);
/// assert!(r.coupling_slope.is_none());
///
/// // last_n larger than slice → silently uses all.
/// let r2 = compute_trend(&[], Some(100));
/// assert_eq!(r2.snapshot_count, 0);
/// ```
pub fn compute_trend(snapshots: &[Snapshot], last_n: Option<usize>) -> TrendResult {
    let window = match last_n {
        None => snapshots,
        Some(n) => {
            let start = snapshots.len().saturating_sub(n);
            &snapshots[start..]
        }
    };

    let count = window.len();

    if count < 2 {
        return TrendResult {
            snapshot_count: count,
            pattern_entropy_slope: None,
            convention_drift_slope: None,
            coupling_slope: None,
            community_count_slope: None,
        };
    }

    use sdivi_patterns::compute_entropy;

    let n_intervals = (count - 1) as f64;

    let entropy_vals: Vec<f64> = window
        .iter()
        .map(|s| s.catalog.entries.values().map(compute_entropy).sum())
        .collect();

    let convention_vals: Vec<f64> = window
        .iter()
        .map(|s| s.pattern_metrics.convention_drift)
        .collect();

    let density_vals: Vec<f64> = window.iter().map(|s| s.graph.density).collect();

    let community_vals: Vec<i64> = window
        .iter()
        .map(|s| s.partition.community_count() as i64)
        .collect();

    let entropy_slope = mean_slope(&entropy_vals);
    let drift_slope = mean_slope(&convention_vals);
    let coupling_slope = mean_slope(&density_vals);
    let community_slope: f64 = community_vals
        .windows(2)
        .map(|w| (w[1] - w[0]) as f64)
        .sum::<f64>()
        / n_intervals;

    TrendResult {
        snapshot_count: count,
        pattern_entropy_slope: Some(entropy_slope),
        convention_drift_slope: Some(drift_slope),
        coupling_slope: Some(coupling_slope),
        community_count_slope: Some(community_slope),
    }
}

fn mean_slope(vals: &[f64]) -> f64 {
    if vals.len() < 2 {
        return 0.0;
    }
    let n = (vals.len() - 1) as f64;
    vals.windows(2).map(|w| w[1] - w[0]).sum::<f64>() / n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{assemble_snapshot, PatternMetricsResult};
    use sdivi_detection::partition::LeidenPartition;
    use sdivi_graph::metrics::GraphMetrics;
    use sdivi_patterns::PatternCatalog;
    use std::collections::BTreeMap;

    fn make_snap(density: f64, communities: usize) -> Snapshot {
        let mut stability = BTreeMap::new();
        for i in 0..communities {
            stability.insert(i, 1.0_f64);
        }
        let graph = GraphMetrics {
            node_count: 2,
            edge_count: 0,
            density,
            cycle_count: 0,
            top_hubs: vec![],
            component_count: 1,
        };
        let partition = LeidenPartition {
            assignments: BTreeMap::new(),
            stability,
            modularity: 0.0,
            seed: 42,
        };
        assemble_snapshot(
            graph,
            partition,
            PatternCatalog::default(),
            PatternMetricsResult::default(),
            None,
            "T",
            None,
            None,
            0,
        )
    }

    #[test]
    fn empty_slice_returns_zero_count() {
        let r = compute_trend(&[], None);
        assert_eq!(r.snapshot_count, 0);
        assert!(r.coupling_slope.is_none());
    }

    #[test]
    fn single_snapshot_no_slopes() {
        let snaps = vec![make_snap(0.5, 3)];
        let r = compute_trend(&snaps, None);
        assert_eq!(r.snapshot_count, 1);
        assert!(r.coupling_slope.is_none());
    }

    #[test]
    fn two_snapshots_correct_slope() {
        let snaps = vec![make_snap(0.1, 2), make_snap(0.3, 4)];
        let r = compute_trend(&snaps, None);
        assert_eq!(r.snapshot_count, 2);
        let slope = r.coupling_slope.unwrap();
        assert!((slope - 0.2).abs() < 1e-10);
        assert_eq!(r.community_count_slope, Some(2.0));
    }

    #[test]
    fn last_n_clamped_to_available() {
        let snaps = vec![make_snap(0.1, 2), make_snap(0.2, 3)];
        // last_n = 100 > snaps.len() → use all 2
        let r = compute_trend(&snaps, Some(100));
        assert_eq!(r.snapshot_count, 2);
    }

    #[test]
    fn last_n_selects_tail() {
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
}
