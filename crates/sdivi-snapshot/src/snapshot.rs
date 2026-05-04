//! [`Snapshot`] — versioned snapshot of pipeline stage outputs.

use std::collections::BTreeMap;

use sdivi_detection::partition::LeidenPartition;
use sdivi_graph::metrics::GraphMetrics;
use sdivi_patterns::PatternCatalog;
use serde::{Deserialize, Serialize};

use crate::change_coupling::ChangeCouplingResult;

/// Snapshot schema version emitted by sdivi-rust.
///
/// This constant is `"1.0"` for all sdivi-rust output.  Bumping this value is a
/// breaking change (Rule 16).
pub const SNAPSHOT_VERSION: &str = "1.0";

/// Intent-divergence summary derived from the caller's boundary representation.
///
/// Present in a [`Snapshot`] only when the caller supplied a boundary count to
/// [`assemble_snapshot`] (typically because a `.sdivi/boundaries.yaml` was
/// found at snapshot time, but any source of a count is valid).
///
/// # Examples
///
/// ```rust
/// use sdivi_snapshot::snapshot::IntentDivergenceInfo;
///
/// let info = IntentDivergenceInfo { boundary_count: 3, violation_count: 0 };
/// assert_eq!(info.boundary_count, 3);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntentDivergenceInfo {
    /// Number of boundaries declared by the caller.
    pub boundary_count: usize,
    /// Number of cross-boundary dependency violations detected.
    pub violation_count: u32,
}

/// Pattern metrics derived from the catalog — carried in every snapshot.
///
/// Computed by `sdivi_core::compute_pattern_metrics` or populated by
/// `sdivi_pipeline::Pipeline` from the full catalog.
///
/// # Examples
///
/// ```rust
/// use std::collections::BTreeMap;
/// use sdivi_snapshot::snapshot::PatternMetricsResult;
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
/// use sdivi_snapshot::snapshot::{assemble_snapshot, PatternMetricsResult, SNAPSHOT_VERSION};
/// use sdivi_graph::metrics::GraphMetrics;
/// use sdivi_detection::partition::LeidenPartition;
/// use sdivi_patterns::PatternCatalog;
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
///     "2026-04-29T00:00:00Z", None, None, 0,
/// );
/// assert_eq!(snap.snapshot_version, SNAPSHOT_VERSION);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snapshot {
    /// Always `"1.0"` for sdivi-rust output.
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
    /// Intent divergence against the caller-declared boundaries.
    ///
    /// `None` (omitted from JSON) when no boundary count was supplied to
    /// [`assemble_snapshot`] (typically because `.sdivi/boundaries.yaml` was
    /// not present).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent_divergence: Option<IntentDivergenceInfo>,
    /// File-path → community-ID assignments for boundary inference.
    ///
    /// Maps each source file's repo-relative path to its community ID from the
    /// Leiden partition at snapshot time. Populated by `sdivi-pipeline` from the
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
/// When `boundary_count` is `Some`, an [`IntentDivergenceInfo`] is included with
/// that count and the caller-supplied `violation_count`. The caller is responsible
/// for deriving `boundary_count` from a `BoundarySpec` (or any equivalent source);
/// this function is intentionally agnostic to the spec type so non-FS callers
/// (WASM, embedders with their own boundary representation) can use it directly
/// without constructing a `sdivi_config::BoundarySpec`.
///
/// # Examples
///
/// ```rust
/// use std::collections::BTreeMap;
/// use sdivi_snapshot::snapshot::{assemble_snapshot, PatternMetricsResult, SNAPSHOT_VERSION};
/// use sdivi_graph::metrics::GraphMetrics;
/// use sdivi_detection::partition::LeidenPartition;
/// use sdivi_patterns::PatternCatalog;
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
///     "2026-04-29T00:00:00Z", Some("abc123"), None, 0,
/// );
/// assert_eq!(snap.commit.as_deref(), Some("abc123"));
/// ```
#[allow(clippy::too_many_arguments)] // 9 args: every field is load-bearing; seam between sdivi-pipeline and sdivi-core
pub fn assemble_snapshot(
    graph: GraphMetrics,
    partition: LeidenPartition,
    catalog: PatternCatalog,
    pattern_metrics: PatternMetricsResult,
    boundary_count: Option<usize>,
    timestamp: &str,
    commit: Option<&str>,
    change_coupling: Option<ChangeCouplingResult>,
    violation_count: u32,
) -> Snapshot {
    let intent_divergence = boundary_count.map(|boundary_count| IntentDivergenceInfo {
        boundary_count,
        violation_count,
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
    /// use sdivi_snapshot::snapshot::Snapshot;
    ///
    /// let snap = Snapshot::load(Path::new(".sdivi/snapshots/snapshot_2026.json"));
    /// ```
    #[cfg(feature = "pipeline-records")]
    pub fn load(path: &std::path::Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}
