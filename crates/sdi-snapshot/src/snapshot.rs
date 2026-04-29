//! [`Snapshot`] — versioned snapshot of pipeline stage outputs.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sdi_config::BoundarySpec;
use sdi_detection::partition::LeidenPartition;
use sdi_graph::metrics::GraphMetrics;
use sdi_patterns::PatternCatalog;

/// Snapshot schema version emitted by sdi-rust.
///
/// This constant is `"1.0"` for all sdi-rust output.  Reading a JSON file with
/// a different value produces a stderr warning and baseline treatment (no delta)
/// — never a crash.  Bumping this value is a breaking change (Rule 16).
pub const SNAPSHOT_VERSION: &str = "1.0";

/// Intent-divergence summary computed from a [`BoundarySpec`].
///
/// Present in a [`Snapshot`] only when a `.sdi/boundaries.yaml` file was
/// found at snapshot time.  Absent (serialized as `null` / omitted from JSON)
/// when no boundary spec was loaded.
///
/// # Examples
///
/// ```rust
/// use sdi_snapshot::snapshot::IntentDivergenceInfo;
///
/// let info = IntentDivergenceInfo { boundary_count: 3, violation_count: 0 };
/// assert_eq!(info.boundary_count, 3);
/// assert_eq!(info.violation_count, 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntentDivergenceInfo {
    /// Number of boundaries declared in the [`BoundarySpec`].
    pub boundary_count: usize,
    /// Number of boundary violations detected against the declared spec.
    ///
    /// Always `0` until the violation-detection pass is implemented.
    pub violation_count: u32,
}

/// A versioned snapshot of all pipeline stage outputs for one point in time.
///
/// Snapshots are the durable, serialisable record produced by
/// `Pipeline::snapshot`.  Every field that contributes to delta computation is
/// present and concrete; optional fields are skipped in JSON when absent so the
/// output remains minimal.
///
/// # Examples
///
/// ```rust
/// use std::collections::BTreeMap;
/// use sdi_snapshot::snapshot::{build_snapshot, SNAPSHOT_VERSION};
/// use sdi_graph::metrics::GraphMetrics;
/// use sdi_detection::partition::LeidenPartition;
/// use sdi_patterns::PatternCatalog;
///
/// let graph = GraphMetrics {
///     node_count: 0,
///     edge_count: 0,
///     density: 0.0,
///     cycle_count: 0,
///     top_hubs: vec![],
///     component_count: 0,
/// };
/// let partition = LeidenPartition {
///     assignments: BTreeMap::new(),
///     stability: BTreeMap::new(),
///     modularity: 0.0,
///     seed: 42,
/// };
/// let catalog = PatternCatalog::default();
/// let snap = build_snapshot(graph, partition, catalog, None, "2026-04-29T00:00:00Z", None);
/// assert_eq!(snap.snapshot_version, SNAPSHOT_VERSION);
/// assert!(snap.intent_divergence.is_none());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snapshot {
    /// Always `"1.0"` for sdi-rust output.
    pub snapshot_version: String,

    /// ISO 8601 UTC timestamp at which the snapshot was taken.
    ///
    /// Example: `"2026-04-29T12:34:56Z"`.
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

    /// Intent divergence against the declared [`BoundarySpec`].
    ///
    /// `None` (omitted from JSON) when no `.sdi/boundaries.yaml` was present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent_divergence: Option<IntentDivergenceInfo>,
}

/// Assembles a [`Snapshot`] from pipeline stage outputs.
///
/// When `boundary_spec` is `Some`, an [`IntentDivergenceInfo`] is computed
/// (boundary count from the spec; violation count is `0` until the
/// violation-detection pass is implemented).  When `boundary_spec` is `None`,
/// `intent_divergence` is absent from the snapshot (Rule 16 / Rule 8).
///
/// # Examples
///
/// ```rust
/// use std::collections::BTreeMap;
/// use sdi_snapshot::snapshot::{build_snapshot, SNAPSHOT_VERSION};
/// use sdi_graph::metrics::GraphMetrics;
/// use sdi_detection::partition::LeidenPartition;
/// use sdi_patterns::PatternCatalog;
///
/// let graph = GraphMetrics {
///     node_count: 1,
///     edge_count: 0,
///     density: 0.0,
///     cycle_count: 0,
///     top_hubs: vec![],
///     component_count: 1,
/// };
/// let partition = LeidenPartition {
///     assignments: BTreeMap::from([(0, 0)]),
///     stability: BTreeMap::from([(0, 1.0)]),
///     modularity: 0.0,
///     seed: 42,
/// };
/// let catalog = PatternCatalog::default();
/// let snap = build_snapshot(
///     graph, partition, catalog, None,
///     "2026-04-29T00:00:00Z",
///     Some("abc123"),
/// );
/// assert_eq!(snap.snapshot_version, SNAPSHOT_VERSION);
/// assert_eq!(snap.commit.as_deref(), Some("abc123"));
/// ```
pub fn build_snapshot(
    graph: GraphMetrics,
    partition: LeidenPartition,
    catalog: PatternCatalog,
    boundary_spec: Option<&BoundarySpec>,
    timestamp: &str,
    commit: Option<&str>,
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
        intent_divergence,
    }
}

impl Snapshot {
    /// Loads a [`Snapshot`] from a JSON file at `path`.
    ///
    /// Returns `Err` with [`io::ErrorKind::InvalidData`] if the file exists
    /// but cannot be deserialized.  Other I/O errors (not-found, permission
    /// denied) propagate their original [`io::ErrorKind`].
    ///
    /// # Errors
    ///
    /// Returns [`io::Error`] on read or parse failure.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use sdi_snapshot::snapshot::Snapshot;
    ///
    /// let snap = Snapshot::load(Path::new(".sdi/snapshots/snapshot_2026.json"));
    /// // Returns Err on missing or malformed file.
    /// ```
    pub fn load(path: &Path) -> io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
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
            node_count: 0,
            edge_count: 0,
            density: 0.0,
            cycle_count: 0,
            top_hubs: vec![],
            component_count: 0,
        }
    }

    fn empty_partition() -> LeidenPartition {
        LeidenPartition {
            assignments: BTreeMap::new(),
            stability: BTreeMap::new(),
            modularity: 0.0,
            seed: 42,
        }
    }

    #[test]
    fn build_snapshot_sets_version() {
        let snap =
            build_snapshot(empty_graph(), empty_partition(), PatternCatalog::default(), None, "T", None);
        assert_eq!(snap.snapshot_version, SNAPSHOT_VERSION);
    }

    #[test]
    fn no_boundary_spec_means_no_intent_divergence() {
        let snap =
            build_snapshot(empty_graph(), empty_partition(), PatternCatalog::default(), None, "T", None);
        assert!(snap.intent_divergence.is_none());
    }

    #[test]
    fn commit_round_trips() {
        let snap = build_snapshot(
            empty_graph(),
            empty_partition(),
            PatternCatalog::default(),
            None,
            "T",
            Some("deadbeef"),
        );
        assert_eq!(snap.commit.as_deref(), Some("deadbeef"));
    }

    #[test]
    fn serde_round_trip() {
        let snap =
            build_snapshot(empty_graph(), empty_partition(), PatternCatalog::default(), None, "T", None);
        let json = serde_json::to_string(&snap).unwrap();
        let decoded: Snapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snap, decoded);
    }

    #[test]
    fn commit_none_absent_from_json() {
        let snap =
            build_snapshot(empty_graph(), empty_partition(), PatternCatalog::default(), None, "T", None);
        let json = serde_json::to_string(&snap).unwrap();
        assert!(!json.contains("\"commit\""));
    }
}
