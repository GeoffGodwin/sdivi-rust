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
/// use sdivi_core::input::NodeInput;
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
/// use sdivi_core::input::EdgeInput;
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
/// use sdivi_core::input::{DependencyGraphInput, NodeInput, EdgeInput};
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
/// use sdivi_core::input::PatternLocationInput;
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
/// use sdivi_core::input::PatternInstanceInput;
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
/// use sdivi_core::input::{LeidenConfigInput, QualityFunctionInput};
///
/// let cfg = LeidenConfigInput::default();
/// assert_eq!(cfg.seed, 42);
/// assert_eq!(cfg.min_compression_ratio, 0.1);
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
    /// Per-edge weights for weighted Leiden; `None` = 1.0 for all edges.
    /// Keys: `"source\x00target"` (NUL-sep, `source < target`); use [`crate::input::edge_weight_key`].
    #[serde(default)]
    pub edge_weights: Option<BTreeMap<String, f64>>,
    /// Stop recursive Leiden when an aggregation level would compress the graph
    /// by less than this fraction of nodes. Must be in `[0.0, 1.0)`. Default `0.1`.
    #[serde(default = "default_min_compression_ratio")]
    pub min_compression_ratio: f64,
    /// Hard cap on Leiden recursion depth. Default `32`.
    #[serde(default = "default_max_recursion_depth")]
    pub max_recursion_depth: u32,
}

fn default_min_compression_ratio() -> f64 {
    0.1
}

fn default_max_recursion_depth() -> u32 {
    32
}

impl Default for LeidenConfigInput {
    fn default() -> Self {
        LeidenConfigInput {
            seed: 42,
            gamma: 1.0,
            iterations: 100,
            quality: QualityFunctionInput::Modularity,
            edge_weights: None,
            min_compression_ratio: 0.1,
            max_recursion_depth: 32,
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
/// use sdivi_core::input::PriorPartition;
/// use std::collections::BTreeMap;
///
/// let p = PriorPartition { cluster_assignments: BTreeMap::new() };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PriorPartition {
    /// Node ID → community ID mapping.
    pub cluster_assignments: BTreeMap<String, u32>,
}

// ── Normalize input ──────────────────────────────────────────────────────────

/// A node in the pattern AST subtree for [`crate::normalize_and_hash`].
///
/// # Examples
///
/// ```rust
/// use sdivi_core::input::NormalizeNode;
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
/// use sdivi_core::input::BoundarySpecInput;
///
/// let spec = BoundarySpecInput { boundaries: vec![] };
/// assert!(spec.boundaries.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundarySpecInput {
    /// Declared boundaries.
    pub boundaries: Vec<BoundaryDefInput>,
}
