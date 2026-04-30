//! Input struct definitions for the pure-compute API.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// ── Graph inputs ─────────────────────────────────────────────────────────────

/// A single node in a [`DependencyGraphInput`].
///
/// `id` must satisfy the [`super::validate_node_id`] rules.
///
/// # Examples
///
/// ```rust
/// use sdi_core::input::NodeInput;
///
/// let n = NodeInput {
///     id: "src/lib.rs".to_string(),
///     path: "src/lib.rs".to_string(),
///     language: "rust".to_string(),
/// };
/// assert_eq!(n.id, "src/lib.rs");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeInput {
    /// Canonical node ID: repo-relative path, forward slashes, no leading
    /// `./`, no trailing `/`, not absolute, no `..`, not empty.
    pub id: String,
    /// Human-readable path (may differ from `id` if aliased).
    pub path: String,
    /// Language tag (e.g., `"rust"`, `"python"`).
    pub language: String,
}

/// A directed edge between two nodes in a [`DependencyGraphInput`].
///
/// `source` and `target` reference `NodeInput::id` values.
///
/// # Examples
///
/// ```rust
/// use sdi_core::input::EdgeInput;
///
/// let e = EdgeInput { source: "src/lib.rs".to_string(), target: "src/models.rs".to_string() };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EdgeInput {
    /// Source node ID.
    pub source: String,
    /// Target node ID.
    pub target: String,
}

/// Input graph for pure-compute functions.
///
/// # Examples
///
/// ```rust
/// use sdi_core::input::{DependencyGraphInput, NodeInput, EdgeInput};
///
/// let g = DependencyGraphInput { nodes: vec![], edges: vec![] };
/// assert_eq!(g.nodes.len(), 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DependencyGraphInput {
    /// All nodes; order determines 0-based numeric indices used in edges.
    pub nodes: Vec<NodeInput>,
    /// All directed edges referencing node IDs.
    pub edges: Vec<EdgeInput>,
}

// ── Pattern inputs ────────────────────────────────────────────────────────────

/// Location of a pattern instance within a source file.
///
/// # Examples
///
/// ```rust
/// use sdi_core::input::PatternLocationInput;
///
/// let loc = PatternLocationInput { file: "src/lib.rs".to_string(), start_row: 5, start_col: 0 };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatternLocationInput {
    /// Repo-relative file path (string, not PathBuf, for tsify compat).
    pub file: String,
    /// Zero-indexed source row.
    pub start_row: u32,
    /// Zero-indexed source column.
    pub start_col: u32,
}

/// A single pattern instance supplied by a foreign extractor.
///
/// # Examples
///
/// ```rust
/// use sdi_core::input::PatternInstanceInput;
///
/// let p = PatternInstanceInput {
///     fingerprint: "abc".to_string(),
///     category: "error_handling".to_string(),
///     node_id: "src/lib.rs".to_string(),
///     location: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatternInstanceInput {
    /// 64-char lowercase hex fingerprint string.
    pub fingerprint: String,
    /// Pattern category (e.g., `"error_handling"`).
    pub category: String,
    /// Node ID of the source file containing this instance.
    pub node_id: String,
    /// Source position (optional — omit when position is unknown).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<PatternLocationInput>,
}

// ── Detection inputs ─────────────────────────────────────────────────────────

/// Leiden algorithm configuration for pure-compute callers.
///
/// # Examples
///
/// ```rust
/// use sdi_core::input::{LeidenConfigInput, QualityFunctionInput};
///
/// let cfg = LeidenConfigInput::default();
/// assert_eq!(cfg.seed, 42);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LeidenConfigInput {
    /// Random seed for deterministic results.
    pub seed: u64,
    /// Resolution parameter (CPM only; ignored for Modularity).
    pub gamma: f64,
    /// Maximum outer-loop iterations.
    pub iterations: usize,
    /// Quality function to optimise.
    pub quality: QualityFunctionInput,
}

impl Default for LeidenConfigInput {
    fn default() -> Self {
        LeidenConfigInput {
            seed: 42,
            gamma: 1.0,
            iterations: 100,
            quality: QualityFunctionInput::Modularity,
        }
    }
}

/// Quality function selection for [`LeidenConfigInput`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QualityFunctionInput {
    /// Newman–Girvan modularity.
    Modularity,
    /// Constant Potts Model.
    Cpm,
}

/// A prior partition snapshot for consecutive-snapshot stability scoring.
///
/// # Examples
///
/// ```rust
/// use sdi_core::input::PriorPartition;
/// use std::collections::BTreeMap;
///
/// let p = PriorPartition { cluster_assignments: BTreeMap::new() };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PriorPartition {
    /// Node ID → community ID mapping.
    pub cluster_assignments: BTreeMap<String, u32>,
}

// ── Normalize input ───────────────────────────────────────────────────────────

/// A node in the pattern AST subtree for [`crate::normalize_and_hash`].
///
/// # Examples
///
/// ```rust
/// use sdi_core::input::NormalizeNode;
///
/// let leaf = NormalizeNode { kind: "try_expression".to_string(), children: vec![] };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NormalizeNode {
    /// Tree-sitter node kind.
    pub kind: String,
    /// Ordered children.
    pub children: Vec<NormalizeNode>,
}

// ── Threshold inputs ──────────────────────────────────────────────────────────

/// Per-category threshold override for [`ThresholdsInput`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThresholdOverrideInput {
    /// Overridden pattern entropy rate (uses default if absent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_entropy_rate: Option<f64>,
    /// Overridden convention drift rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub convention_drift_rate: Option<f64>,
    /// Overridden coupling delta rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coupling_delta_rate: Option<f64>,
    /// Overridden boundary violation rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boundary_violation_rate: Option<f64>,
    /// ISO-8601 expiry date (`"YYYY-MM-DD"`).
    pub expires: String,
}

/// Threshold configuration for [`crate::compute_thresholds_check`].
///
/// The caller supplies `today` explicitly — no clock access in sdi-core.
///
/// **IMPORTANT:** `ThresholdsInput::default()` sets `today` to a far-future
/// sentinel (`9999-12-31`) so that all per-category overrides are treated as
/// expired (i.e., the global rates apply).  Callers that use per-category
/// overrides MUST supply the real current date:
///
/// ```rust
/// # use chrono::NaiveDate;
/// use sdi_core::input::ThresholdsInput;
///
/// let today = NaiveDate::from_ymd_opt(2026, 4, 30).unwrap();
/// let t = ThresholdsInput { today, ..ThresholdsInput::default() };
/// assert_eq!(t.pattern_entropy_rate, 2.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThresholdsInput {
    /// Maximum allowed pattern entropy rate.
    pub pattern_entropy_rate: f64,
    /// Maximum allowed convention drift rate.
    pub convention_drift_rate: f64,
    /// Maximum allowed coupling delta rate.
    pub coupling_delta_rate: f64,
    /// Maximum allowed boundary violation rate.
    pub boundary_violation_rate: f64,
    /// Per-category overrides (may include expired entries — `today` determines which apply).
    #[serde(default)]
    pub overrides: BTreeMap<String, ThresholdOverrideInput>,
    /// Today's date for expiry evaluation.  Caller supplies this (no clock in sdi-core).
    /// `Default` uses `9999-12-31`; override with the real date to enable per-category filtering.
    pub today: chrono::NaiveDate,
}

impl Default for ThresholdsInput {
    fn default() -> Self {
        ThresholdsInput {
            pattern_entropy_rate: 2.0,
            convention_drift_rate: 3.0,
            coupling_delta_rate: 0.15,
            boundary_violation_rate: 2.0,
            overrides: BTreeMap::new(),
            // Far-future sentinel — callers must supply the real `today` to enable override filtering.
            today: chrono::NaiveDate::from_ymd_opt(9999, 12, 31).unwrap(),
        }
    }
}

// ── Boundary inputs ───────────────────────────────────────────────────────────

/// A single boundary definition for [`BoundarySpecInput`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundaryDefInput {
    /// Boundary name.
    pub name: String,
    /// Glob patterns for files in this boundary.
    pub modules: Vec<String>,
    /// Boundaries this one is allowed to import from.
    pub allow_imports_from: Vec<String>,
}

/// Boundary specification for [`crate::compute_boundary_violations`].
///
/// # Examples
///
/// ```rust
/// use sdi_core::input::BoundarySpecInput;
///
/// let spec = BoundarySpecInput { boundaries: vec![] };
/// assert!(spec.boundaries.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundarySpecInput {
    /// Declared boundaries.
    pub boundaries: Vec<BoundaryDefInput>,
}
