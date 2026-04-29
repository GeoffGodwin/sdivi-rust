//! [`DivergenceSummary`] and [`compute_delta`] — pure delta computation.

use serde::{Deserialize, Serialize};
use sdi_patterns::compute_entropy;

use crate::snapshot::Snapshot;

/// Per-dimension divergence between two consecutive [`Snapshot`]s.
///
/// All fields are `Option<_>`.  `None` means the dimension could not be
/// compared (e.g. one snapshot lacked the prerequisite data).  `Some(0)`
/// or `Some(0.0)` means the dimension was compared and no change was
/// observed.  These two states are intentionally distinct (Rule 14 / KDD-9).
///
/// Null fields are serialised as explicit JSON `null` — `skip_serializing_if`
/// is intentionally absent — so that CI consumers can distinguish "not
/// computed" from "zero change."
///
/// # Examples
///
/// ```rust
/// use sdi_snapshot::delta::null_summary;
///
/// let s = null_summary();
/// assert!(s.pattern_entropy_delta.is_none());
/// assert!(s.coupling_delta.is_none());
/// assert!(s.community_count_delta.is_none());
/// assert!(s.boundary_violation_delta.is_none());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DivergenceSummary {
    /// Change in total normalized pattern entropy (curr − prev).
    ///
    /// Computed as the sum over all categories of `compute_entropy(cat_stats)`,
    /// then `curr_total − prev_total`.  `None` when either snapshot has no
    /// catalog entries.
    pub pattern_entropy_delta: Option<f64>,

    /// Change in graph density (curr − prev).
    ///
    /// `density` is `edge_count / (node_count * (node_count − 1))` and is
    /// always in `[0.0, 1.0]`.
    pub coupling_delta: Option<f64>,

    /// Change in community count (curr − prev), as a signed integer.
    pub community_count_delta: Option<i64>,

    /// Change in boundary violation count (curr − prev), as a signed integer.
    ///
    /// `None` when either snapshot is missing `intent_divergence` (i.e. no
    /// boundary spec was loaded at snapshot time).
    pub boundary_violation_delta: Option<i64>,
}

/// Returns a [`DivergenceSummary`] with all fields `None`.
///
/// Used for the first-snapshot case where no previous snapshot exists.
/// Callers must never substitute `0` for `None` here — `None` explicitly
/// means "no prior snapshot to compare" (Rule 14).
///
/// # Examples
///
/// ```rust
/// use sdi_snapshot::delta::null_summary;
///
/// let s = null_summary();
/// assert_eq!(s, sdi_snapshot::delta::DivergenceSummary {
///     pattern_entropy_delta: None,
///     coupling_delta: None,
///     community_count_delta: None,
///     boundary_violation_delta: None,
/// });
/// ```
pub fn null_summary() -> DivergenceSummary {
    DivergenceSummary {
        pattern_entropy_delta: None,
        coupling_delta: None,
        community_count_delta: None,
        boundary_violation_delta: None,
    }
}

/// Computes the per-dimension divergence between `prev` and `curr`.
///
/// This function is **referentially transparent**: same inputs always produce
/// the same output.  It performs no I/O, reads no globals, and uses no clock.
///
/// ## Dimension computations
///
/// - `pattern_entropy_delta`: sum of `compute_entropy(cat)` across all
///   categories in each snapshot's catalog, then `curr − prev`.
/// - `coupling_delta`: `curr.graph.density − prev.graph.density`.
/// - `community_count_delta`: `curr.partition.community_count() as i64
///   − prev.partition.community_count() as i64`.
/// - `boundary_violation_delta`: `Some(curr_violations − prev_violations)`
///   only when **both** snapshots contain `intent_divergence`; otherwise `None`.
///
/// # Examples
///
/// ```rust
/// use std::collections::BTreeMap;
/// use sdi_snapshot::snapshot::{build_snapshot, Snapshot};
/// use sdi_snapshot::delta::compute_delta;
/// use sdi_graph::metrics::GraphMetrics;
/// use sdi_detection::partition::LeidenPartition;
/// use sdi_patterns::PatternCatalog;
///
/// fn make_snap(density: f64, communities: usize) -> Snapshot {
///     let mut stability = BTreeMap::new();
///     for i in 0..communities {
///         stability.insert(i, 1.0);
///     }
///     let graph = GraphMetrics {
///         node_count: 2,
///         edge_count: 0,
///         density,
///         cycle_count: 0,
///         top_hubs: vec![],
///         component_count: 1,
///     };
///     let partition = LeidenPartition {
///         assignments: BTreeMap::new(),
///         stability,
///         modularity: 0.0,
///         seed: 42,
///     };
///     build_snapshot(graph, partition, PatternCatalog::default(), None, "T", None)
/// }
///
/// let prev = make_snap(0.1, 2);
/// let curr = make_snap(0.3, 3);
/// let delta = compute_delta(&prev, &curr);
/// assert!((delta.coupling_delta.unwrap() - 0.2).abs() < 1e-10);
/// assert_eq!(delta.community_count_delta, Some(1));
/// ```
pub fn compute_delta(prev: &Snapshot, curr: &Snapshot) -> DivergenceSummary {
    let pattern_entropy_delta = Some({
        let prev_entropy: f64 = prev
            .catalog
            .entries
            .values()
            .map(|cat| compute_entropy(cat))
            .sum();
        let curr_entropy: f64 = curr
            .catalog
            .entries
            .values()
            .map(|cat| compute_entropy(cat))
            .sum();
        curr_entropy - prev_entropy
    });

    let coupling_delta = Some(curr.graph.density - prev.graph.density);

    let community_count_delta = Some(
        curr.partition.community_count() as i64 - prev.partition.community_count() as i64,
    );

    let boundary_violation_delta = match (&prev.intent_divergence, &curr.intent_divergence) {
        (Some(p), Some(c)) => {
            Some(i64::from(c.violation_count) - i64::from(p.violation_count))
        }
        _ => None,
    };

    DivergenceSummary {
        pattern_entropy_delta,
        coupling_delta,
        community_count_delta,
        boundary_violation_delta,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use sdi_detection::partition::LeidenPartition;
    use sdi_graph::metrics::GraphMetrics;
    use sdi_patterns::PatternCatalog;

    use super::*;
    use crate::snapshot::{IntentDivergenceInfo, build_snapshot};

    fn make_graph(density: f64) -> GraphMetrics {
        GraphMetrics {
            node_count: 2,
            edge_count: 0,
            density,
            cycle_count: 0,
            top_hubs: vec![],
            component_count: 1,
        }
    }

    fn make_partition(community_count: usize) -> LeidenPartition {
        let mut stability = BTreeMap::new();
        for i in 0..community_count {
            stability.insert(i, 1.0_f64);
        }
        LeidenPartition {
            assignments: BTreeMap::new(),
            stability,
            modularity: 0.0,
            seed: 42,
        }
    }

    fn make_snap(density: f64, communities: usize) -> Snapshot {
        build_snapshot(
            make_graph(density),
            make_partition(communities),
            PatternCatalog::default(),
            None,
            "T",
            None,
        )
    }

    #[test]
    fn null_summary_all_none() {
        let s = null_summary();
        assert!(s.pattern_entropy_delta.is_none());
        assert!(s.coupling_delta.is_none());
        assert!(s.community_count_delta.is_none());
        assert!(s.boundary_violation_delta.is_none());
    }

    #[test]
    fn coupling_delta_correct() {
        let prev = make_snap(0.1, 2);
        let curr = make_snap(0.3, 2);
        let d = compute_delta(&prev, &curr);
        let v = d.coupling_delta.unwrap();
        assert!((v - 0.2).abs() < 1e-10, "expected ~0.2, got {v}");
    }

    #[test]
    fn community_count_delta_correct() {
        let prev = make_snap(0.0, 3);
        let curr = make_snap(0.0, 5);
        let d = compute_delta(&prev, &curr);
        assert_eq!(d.community_count_delta, Some(2));
    }

    #[test]
    fn boundary_violation_delta_none_when_both_missing() {
        let prev = make_snap(0.0, 1);
        let curr = make_snap(0.0, 1);
        let d = compute_delta(&prev, &curr);
        assert!(d.boundary_violation_delta.is_none());
    }

    #[test]
    fn boundary_violation_delta_none_when_one_missing() {
        let mut prev = make_snap(0.0, 1);
        prev.intent_divergence = Some(IntentDivergenceInfo { boundary_count: 2, violation_count: 1 });
        let curr = make_snap(0.0, 1);
        let d = compute_delta(&prev, &curr);
        assert!(d.boundary_violation_delta.is_none());
    }

    #[test]
    fn boundary_violation_delta_computed_when_both_present() {
        let mut prev = make_snap(0.0, 1);
        prev.intent_divergence = Some(IntentDivergenceInfo { boundary_count: 2, violation_count: 1 });
        let mut curr = make_snap(0.0, 1);
        curr.intent_divergence = Some(IntentDivergenceInfo { boundary_count: 2, violation_count: 3 });
        let d = compute_delta(&prev, &curr);
        assert_eq!(d.boundary_violation_delta, Some(2));
    }

    #[test]
    fn pattern_entropy_delta_zero_for_empty_catalogs() {
        let prev = make_snap(0.0, 1);
        let curr = make_snap(0.0, 1);
        let d = compute_delta(&prev, &curr);
        assert_eq!(d.pattern_entropy_delta, Some(0.0));
    }

    #[test]
    fn null_summary_serde_produces_explicit_nulls() {
        let s = null_summary();
        let json = serde_json::to_string(&s).unwrap();
        // All four fields must appear as explicit null in JSON.
        assert!(json.contains("\"pattern_entropy_delta\":null"));
        assert!(json.contains("\"coupling_delta\":null"));
        assert!(json.contains("\"community_count_delta\":null"));
        assert!(json.contains("\"boundary_violation_delta\":null"));
    }
}
