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
/// assert!(s.convention_drift_delta.is_none());
/// assert!(s.pattern_entropy_per_category_delta.is_none());
/// assert!(s.convention_drift_per_category_delta.is_none());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DivergenceSummary {
    /// Change in total normalized pattern entropy (curr − prev).
    pub pattern_entropy_delta: Option<f64>,

    /// Change in `convention_drift` (curr − prev).
    ///
    /// `None` when either snapshot has no catalog entries.
    pub convention_drift_delta: Option<f64>,

    /// Change in graph density (curr − prev).
    pub coupling_delta: Option<f64>,

    /// Change in community count (curr − prev), as a signed integer.
    pub community_count_delta: Option<i64>,

    /// Change in boundary violation count (curr − prev).
    ///
    /// `None` when either snapshot is missing `intent_divergence`.
    pub boundary_violation_delta: Option<i64>,

    /// Per-category Shannon entropy delta (curr − prev), keyed by category name.
    ///
    /// `None` on the first-snapshot path (KDD-9).  Map keys are the union of
    /// categories in `prev` and `curr`; missing-side values are treated as `0.0`.
    #[serde(default)]
    pub pattern_entropy_per_category_delta: Option<std::collections::BTreeMap<String, f64>>,

    /// Per-category convention-drift delta (curr − prev), keyed by category name.
    ///
    /// `None` on the first-snapshot path (KDD-9).  Same union-of-keys semantics as
    /// `pattern_entropy_per_category_delta`.
    #[serde(default)]
    pub convention_drift_per_category_delta: Option<std::collections::BTreeMap<String, f64>>,
}

/// Returns a [`DivergenceSummary`] with all fields `None`.
///
/// Used for the first-snapshot case where no previous snapshot exists.
/// Callers must never substitute `0` for `None` here (Rule 14).
///
/// # Examples
///
/// ```rust
/// use sdi_snapshot::delta::null_summary;
///
/// let s = null_summary();
/// assert_eq!(s.pattern_entropy_delta, None);
/// assert_eq!(s.convention_drift_delta, None);
/// assert_eq!(s.coupling_delta, None);
/// assert_eq!(s.community_count_delta, None);
/// assert_eq!(s.boundary_violation_delta, None);
/// ```
pub fn null_summary() -> DivergenceSummary {
    DivergenceSummary {
        pattern_entropy_delta: None,
        convention_drift_delta: None,
        coupling_delta: None,
        community_count_delta: None,
        boundary_violation_delta: None,
        pattern_entropy_per_category_delta: None,
        convention_drift_per_category_delta: None,
    }
}

/// Computes the per-dimension divergence between `prev` and `curr`.
///
/// This function is **referentially transparent**: same inputs always produce
/// the same output.  It performs no I/O, reads no globals, and uses no clock.
///
/// # Examples
///
/// ```rust
/// use std::collections::BTreeMap;
/// use sdi_snapshot::snapshot::{assemble_snapshot, PatternMetricsResult, Snapshot};
/// use sdi_snapshot::delta::compute_delta;
/// use sdi_graph::metrics::GraphMetrics;
/// use sdi_detection::partition::LeidenPartition;
/// use sdi_patterns::PatternCatalog;
///
/// fn make_snap(density: f64) -> Snapshot {
///     let graph = GraphMetrics {
///         node_count: 2, edge_count: 0, density,
///         cycle_count: 0, top_hubs: vec![], component_count: 1,
///     };
///     let partition = LeidenPartition {
///         assignments: BTreeMap::new(), stability: BTreeMap::new(),
///         modularity: 0.0, seed: 42,
///     };
///     assemble_snapshot(graph, partition, PatternCatalog::default(),
///         PatternMetricsResult::default(), None, "T", None)
/// }
///
/// let delta = compute_delta(&make_snap(0.1), &make_snap(0.3));
/// assert!((delta.coupling_delta.unwrap() - 0.2).abs() < 1e-10);
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

    let convention_drift_delta = Some(
        curr.pattern_metrics.convention_drift - prev.pattern_metrics.convention_drift,
    );

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

    let pattern_entropy_per_category_delta = Some(
        delta_per_category(
            &prev.pattern_metrics.entropy_per_category,
            &curr.pattern_metrics.entropy_per_category,
        )
    );

    let convention_drift_per_category_delta = Some(
        delta_per_category(
            &prev.pattern_metrics.convention_drift_per_category,
            &curr.pattern_metrics.convention_drift_per_category,
        )
    );

    DivergenceSummary {
        pattern_entropy_delta,
        convention_drift_delta,
        coupling_delta,
        community_count_delta,
        boundary_violation_delta,
        pattern_entropy_per_category_delta,
        convention_drift_per_category_delta,
    }
}

/// Computes the per-category delta as `curr − prev` over the union of keys.
///
/// Categories present in only one snapshot are treated as `0.0` on the
/// missing side, so a newly-introduced category surfaces as a positive delta.
fn delta_per_category(
    prev: &std::collections::BTreeMap<String, f64>,
    curr: &std::collections::BTreeMap<String, f64>,
) -> std::collections::BTreeMap<String, f64> {
    let mut result = std::collections::BTreeMap::new();
    for key in prev.keys().chain(curr.keys()) {
        if !result.contains_key(key) {
            let p = prev.get(key).copied().unwrap_or(0.0);
            let c = curr.get(key).copied().unwrap_or(0.0);
            result.insert(key.clone(), c - p);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use sdi_detection::partition::LeidenPartition;
    use sdi_graph::metrics::GraphMetrics;
    use sdi_patterns::PatternCatalog;

    use super::*;
    use crate::snapshot::{IntentDivergenceInfo, PatternMetricsResult, assemble_snapshot};

    fn make_snap(density: f64, communities: usize) -> Snapshot {
        let mut stability = BTreeMap::new();
        for i in 0..communities {
            stability.insert(i, 1.0_f64);
        }
        let graph = GraphMetrics {
            node_count: 2, edge_count: 0, density,
            cycle_count: 0, top_hubs: vec![], component_count: 1,
        };
        let partition = LeidenPartition {
            assignments: BTreeMap::new(), stability,
            modularity: 0.0, seed: 42,
        };
        assemble_snapshot(
            graph, partition, PatternCatalog::default(),
            PatternMetricsResult::default(), None, "T", None,
        )
    }

    #[test]
    fn null_summary_all_none() {
        let s = null_summary();
        assert!(s.pattern_entropy_delta.is_none());
        assert!(s.convention_drift_delta.is_none());
        assert!(s.coupling_delta.is_none());
        assert!(s.community_count_delta.is_none());
        assert!(s.boundary_violation_delta.is_none());
        assert!(s.pattern_entropy_per_category_delta.is_none());
        assert!(s.convention_drift_per_category_delta.is_none());
    }

    #[test]
    fn coupling_delta_correct() {
        let d = compute_delta(&make_snap(0.1, 2), &make_snap(0.3, 2));
        let v = d.coupling_delta.unwrap();
        assert!((v - 0.2).abs() < 1e-10, "expected ~0.2, got {v}");
    }

    #[test]
    fn community_count_delta_correct() {
        let d = compute_delta(&make_snap(0.0, 3), &make_snap(0.0, 5));
        assert_eq!(d.community_count_delta, Some(2));
    }

    #[test]
    fn convention_drift_delta_zero_for_equal_snapshots() {
        let snap = make_snap(0.0, 1);
        let d = compute_delta(&snap, &snap);
        assert_eq!(d.convention_drift_delta, Some(0.0));
    }

    #[test]
    fn boundary_violation_delta_none_when_both_missing() {
        let d = compute_delta(&make_snap(0.0, 1), &make_snap(0.0, 1));
        assert!(d.boundary_violation_delta.is_none());
    }

    #[test]
    fn null_summary_serde_produces_explicit_nulls() {
        let s = null_summary();
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"pattern_entropy_delta\":null"));
        assert!(json.contains("\"convention_drift_delta\":null"));
        assert!(json.contains("\"coupling_delta\":null"));
        assert!(json.contains("\"community_count_delta\":null"));
        assert!(json.contains("\"boundary_violation_delta\":null"));
        assert!(json.contains("\"pattern_entropy_per_category_delta\":null"));
        assert!(json.contains("\"convention_drift_per_category_delta\":null"));
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
}
