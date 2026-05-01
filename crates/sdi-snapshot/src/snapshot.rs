//! [`Snapshot`] — versioned snapshot of pipeline stage outputs.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sdi_detection::partition::LeidenPartition;
use sdi_graph::metrics::GraphMetrics;
use sdi_patterns::PatternCatalog;

use crate::change_coupling::ChangeCouplingResult;

/// Snapshot schema version emitted by sdi-rust.
///
/// This constant is `"1.0"` for all sdi-rust output.  Bumping this value is a
/// breaking change (Rule 16).
pub const SNAPSHOT_VERSION: &str = "1.0";

/// Intent-divergence summary computed from a `BoundarySpec`.
///
/// Present in a [`Snapshot`] only when a `.sdi/boundaries.yaml` was found at
/// snapshot time.
///
/// # Examples
///
/// ```rust
/// use sdi_snapshot::snapshot::IntentDivergenceInfo;
///
/// let info = IntentDivergenceInfo { boundary_count: 3, violation_count: 0 };
/// assert_eq!(info.boundary_count, 3);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntentDivergenceInfo {
    /// Number of boundaries declared in the `BoundarySpec`.
    pub boundary_count: usize,
    /// Number of boundary violations detected.
    ///
    /// Always `0` until the violation-detection pass is implemented.
    pub violation_count: u32,
}

/// Pattern metrics derived from the catalog — carried in every snapshot.
///
/// Computed by `sdi_core::compute_pattern_metrics` or populated by
/// `sdi_pipeline::Pipeline` from the full catalog.
///
/// # Examples
///
/// ```rust
/// use std::collections::BTreeMap;
/// use sdi_snapshot::snapshot::PatternMetricsResult;
///
/// let m = PatternMetricsResult {
///     entropy_per_category: BTreeMap::new(),
///     total_entropy: 0.0,
///     convention_drift: 0.0,
///     convention_drift_per_category: BTreeMap::new(),
/// };
/// assert_eq!(m.total_entropy, 0.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PatternMetricsResult {
    /// Shannon entropy of pattern fingerprints per category.
    pub entropy_per_category: BTreeMap<String, f64>,
    /// Sum of per-category entropies.
    pub total_entropy: f64,
    /// Average fraction of distinct fingerprints per category (0–1).
    ///
    /// Defined as: for each category, `distinct_fingerprints / total_instances`,
    /// then average across all categories.  `0.0` when no instances exist.
    pub convention_drift: f64,
    /// Per-category fraction of distinct fingerprints: `distinct / total` for each category.
    ///
    /// Source of truth for per-category override filtering in `compute_thresholds_check`.
    /// The scalar `convention_drift` is the average of this map's values.
    #[serde(default)]
    pub convention_drift_per_category: BTreeMap<String, f64>,
}

/// A versioned snapshot of all pipeline stage outputs for one point in time.
///
/// # Examples
///
/// ```rust
/// use std::collections::BTreeMap;
/// use sdi_snapshot::snapshot::{assemble_snapshot, PatternMetricsResult, SNAPSHOT_VERSION};
/// use sdi_graph::metrics::GraphMetrics;
/// use sdi_detection::partition::LeidenPartition;
/// use sdi_patterns::PatternCatalog;
///
/// let graph = GraphMetrics {
///     node_count: 0, edge_count: 0, density: 0.0,
///     cycle_count: 0, top_hubs: vec![], component_count: 0,
/// };
/// let partition = LeidenPartition {
///     assignments: BTreeMap::new(), stability: BTreeMap::new(),
///     modularity: 0.0, seed: 42,
/// };
/// let snap = assemble_snapshot(
///     graph, partition, PatternCatalog::default(),
///     PatternMetricsResult::default(), None,
///     "2026-04-29T00:00:00Z", None, None,
/// );
/// assert_eq!(snap.snapshot_version, SNAPSHOT_VERSION);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snapshot {
    /// Always `"1.0"` for sdi-rust output.
    pub snapshot_version: String,
    /// ISO 8601 UTC timestamp at which the snapshot was taken.
    pub timestamp: String,
    /// Git commit SHA at the time of the snapshot, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// Graph metrics computed from the dependency graph.
    pub graph: GraphMetrics,
    /// Leiden community detection result.
    pub partition: LeidenPartition,
    /// Pattern fingerprint catalog with per-category entropy.
    pub catalog: PatternCatalog,
    /// Pattern metrics (entropy, convention drift) for this snapshot.
    pub pattern_metrics: PatternMetricsResult,
    /// Intent divergence against the declared `BoundarySpec`.
    ///
    /// `None` (omitted from JSON) when no `.sdi/boundaries.yaml` was present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent_divergence: Option<IntentDivergenceInfo>,
    /// File-path → community-ID assignments for boundary inference.
    ///
    /// Maps each source file's repo-relative path to its community ID from the
    /// Leiden partition at snapshot time. Populated by `sdi-pipeline` from the
    /// `DependencyGraph` + `LeidenPartition`. Absent (empty) in snapshots
    /// produced without path context (e.g., pure-compute path); boundary
    /// inference from such snapshots yields no proposals.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub path_partition: BTreeMap<String, u32>,

    /// Change-coupling analysis result.
    ///
    /// `None` when the repo has no git history or `history_depth = 0`.
    /// `#[serde(default)]` ensures M14-era snapshots deserialize as `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_coupling: Option<ChangeCouplingResult>,
}

/// Assembles a [`Snapshot`] from pipeline stage outputs.
///
/// When `boundary_spec_boundaries` is `Some(n)`, an [`IntentDivergenceInfo`]
/// is included with that boundary count and `violation_count = 0` (violation
/// detection is implemented in a later milestone).
///
/// # Examples
///
/// ```rust
/// use std::collections::BTreeMap;
/// use sdi_snapshot::snapshot::{assemble_snapshot, PatternMetricsResult, SNAPSHOT_VERSION};
/// use sdi_graph::metrics::GraphMetrics;
/// use sdi_detection::partition::LeidenPartition;
/// use sdi_patterns::PatternCatalog;
///
/// let graph = GraphMetrics {
///     node_count: 1, edge_count: 0, density: 0.0,
///     cycle_count: 0, top_hubs: vec![], component_count: 1,
/// };
/// let partition = LeidenPartition {
///     assignments: BTreeMap::from([(0, 0)]),
///     stability: BTreeMap::from([(0, 1.0)]),
///     modularity: 0.0, seed: 42,
/// };
/// let snap = assemble_snapshot(
///     graph, partition, PatternCatalog::default(),
///     PatternMetricsResult::default(), None,
///     "2026-04-29T00:00:00Z", Some("abc123"), None,
/// );
/// assert_eq!(snap.commit.as_deref(), Some("abc123"));
/// ```
pub fn assemble_snapshot(
    graph: GraphMetrics,
    partition: LeidenPartition,
    catalog: PatternCatalog,
    pattern_metrics: PatternMetricsResult,
    boundary_spec: Option<&sdi_config::BoundarySpec>,
    timestamp: &str,
    commit: Option<&str>,
    change_coupling: Option<ChangeCouplingResult>,
) -> Snapshot {
    let intent_divergence = boundary_spec.map(|spec| IntentDivergenceInfo {
        boundary_count: spec.boundaries.len(),
        violation_count: 0,
    });

    Snapshot {
        snapshot_version: SNAPSHOT_VERSION.to_string(),
        timestamp: timestamp.to_string(),
        commit: commit.map(str::to_string),
        graph,
        partition,
        catalog,
        pattern_metrics,
        intent_divergence,
        path_partition: BTreeMap::new(),
        change_coupling,
    }
}

impl Snapshot {
    /// Loads a [`Snapshot`] from a JSON file at `path`.
    ///
    /// Only available with the `pipeline-records` feature (default ON).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use sdi_snapshot::snapshot::Snapshot;
    ///
    /// let snap = Snapshot::load(Path::new(".sdi/snapshots/snapshot_2026.json"));
    /// ```
    #[cfg(feature = "pipeline-records")]
    pub fn load(path: &std::path::Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use sdi_detection::partition::LeidenPartition;
    use sdi_graph::metrics::GraphMetrics;
    use sdi_patterns::PatternCatalog;

    use super::*;

    fn empty_graph() -> GraphMetrics {
        GraphMetrics {
            node_count: 0, edge_count: 0, density: 0.0,
            cycle_count: 0, top_hubs: vec![], component_count: 0,
        }
    }

    fn empty_partition() -> LeidenPartition {
        LeidenPartition {
            assignments: BTreeMap::new(), stability: BTreeMap::new(),
            modularity: 0.0, seed: 42,
        }
    }

    fn make_snap() -> Snapshot {
        assemble_snapshot(
            empty_graph(), empty_partition(), PatternCatalog::default(),
            PatternMetricsResult::default(), None, "T", None, None,
        )
    }

    #[test]
    fn assemble_snapshot_sets_version() {
        let snap = make_snap();
        assert_eq!(snap.snapshot_version, SNAPSHOT_VERSION);
    }

    #[test]
    fn no_boundary_spec_means_no_intent_divergence() {
        let snap = make_snap();
        assert!(snap.intent_divergence.is_none());
    }

    #[test]
    fn commit_round_trips() {
        let snap = assemble_snapshot(
            empty_graph(), empty_partition(), PatternCatalog::default(),
            PatternMetricsResult::default(), None, "T", Some("deadbeef"), None,
        );
        assert_eq!(snap.commit.as_deref(), Some("deadbeef"));
    }

    #[test]
    fn serde_round_trip() {
        let snap = make_snap();
        let json = serde_json::to_string(&snap).unwrap();
        let decoded: Snapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snap, decoded);
    }

    #[test]
    fn commit_none_absent_from_json() {
        let snap = make_snap();
        let json = serde_json::to_string(&snap).unwrap();
        assert!(!json.contains("\"commit\""));
    }

    #[test]
    fn pattern_metrics_present_in_json() {
        let snap = make_snap();
        let json = serde_json::to_string(&snap).unwrap();
        assert!(json.contains("\"pattern_metrics\""));
    }
}
