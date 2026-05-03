#![deny(missing_docs)]
//! # sdivi-core
//!
//! Pure-compute facade for the Structural Divergence Indexer (sdivi-rust).
//!
//! This crate is the stable, WASM-compatible public surface for embedding the
//! analysis pipeline in Rust programs, WASM bindings, and language bindings.
//! It has **no I/O, no clock, no tree-sitter, and no `std::fs`** — callers
//! supply pre-extracted data via the `*Input` struct family and receive
//! plain `serde` result types.
//!
//! Embedders that need the full FS-orchestrated pipeline (parsing, snapshot
//! writes, retention) should use `sdivi-pipeline` instead.
//!
//! # Quick start
//!
//! ```rust
//! use sdivi_core::ExitCode;
//!
//! assert_eq!(ExitCode::Success as i32, 0);
//! ```
//!
//! ## Pure-compute example
//!
//! ```rust
//! use sdivi_core::compute::coupling::compute_coupling_topology;
//! use sdivi_core::input::DependencyGraphInput;
//!
//! let g = DependencyGraphInput { nodes: vec![], edges: vec![] };
//! let result = compute_coupling_topology(&g).unwrap();
//! assert_eq!(result.node_count, 0);
//! ```

/// Pattern category contract — canonical category list and runtime discovery via [`list_categories`].
pub mod categories;

/// Errors produced by the sdivi-core pure-compute API.
pub mod error;

/// Exit codes for the `sdivi` binary — public API, adding variants is a breaking change.
pub mod exit_code;

/// Input structs for the pure-compute API (WASM-safe serde types).
pub mod input;

/// Pure-compute functions over [`input`] structs.
pub mod compute;

/// Re-exports of snapshot assembly, delta, trend, and boundary inference from `sdivi-snapshot`.
pub mod facade;

pub use categories::{list_categories, CategoryCatalog, CategoryInfo, CATEGORIES};
pub use error::AnalysisError;
pub use exit_code::ExitCode;

// ── input type re-exports ──────────────────────────────────────────────────

pub use input::{
    edge_weight_key, split_edge_weight_key, validate_node_id, BoundaryDefInput, BoundarySpecInput,
    ChangeCouplingConfigInput, CoChangeEventInput, DependencyGraphInput, EdgeInput,
    LeidenConfigInput, NodeInput, NormalizeNode, PatternInstanceInput, PatternLocationInput,
    PriorPartition, QualityFunctionInput, ThresholdOverrideInput, ThresholdsInput,
};

// ── compute function re-exports ────────────────────────────────────────────

pub use compute::boundaries::{
    compute_boundary_violations, detect_boundaries, BoundaryDetectionResult,
    BoundaryViolationResult,
};
pub use compute::change_coupling::compute_change_coupling;
pub use compute::coupling::{compute_coupling_topology, CouplingTopologyResult};
pub use compute::normalize::normalize_and_hash;
pub use compute::patterns::{compute_pattern_metrics, compute_pattern_metrics_from_catalog};
pub use compute::thresholds::{
    compute_thresholds_check, AppliedOverrideInfo, ThresholdBreachInfo, ThresholdCheckResult,
    THRESHOLD_EPSILON,
};

// ── facade re-exports (sdivi-snapshot) ──────────────────────────────────────

pub use facade::{assemble_snapshot, compute_delta, compute_trend, infer_boundaries, null_summary};

// ── snapshot types re-exported directly ───────────────────────────────────

pub use sdivi_snapshot::boundary_inference::{
    BoundaryInferenceResult, BoundaryProposal, PriorPartition as SnapshotPriorPartition,
};
pub use sdivi_snapshot::change_coupling::{ChangeCouplingResult, CoChangePair};
pub use sdivi_snapshot::delta::DivergenceSummary;
pub use sdivi_snapshot::snapshot::{
    IntentDivergenceInfo, PatternMetricsResult, Snapshot, SNAPSHOT_VERSION,
};
pub use sdivi_snapshot::trend::TrendResult;

// ── fingerprint key re-export ──────────────────────────────────────────────

/// The fixed `blake3` key used for all pattern fingerprints.
///
/// Foreign extractors that produce pattern fingerprints must use this same key
/// to ensure their fingerprints are byte-identical to those produced by the
/// native Rust pipeline.
///
/// See [`normalize_and_hash`] for the canonical tree-aware algorithm.
pub use sdivi_patterns::FINGERPRINT_KEY;

// ── inner-crate type re-exports (for sdivi-wasm and other embedders) ─────────

/// Graph metrics summary — re-exported from `sdivi-graph` for WASM embedders.
pub use sdivi_graph::metrics::GraphMetrics;

/// Leiden community detection result — re-exported from `sdivi-detection` for WASM embedders.
pub use sdivi_detection::partition::LeidenPartition;

/// Pattern fingerprint catalog — re-exported from `sdivi-patterns` for WASM embedders.
pub use sdivi_patterns::catalog::{PatternCatalog, PatternStats};

/// Pattern fingerprint type — re-exported from `sdivi-patterns` for WASM embedders.
pub use sdivi_patterns::fingerprint::PatternFingerprint;

/// Commonly-imported items from sdivi-core.
pub mod prelude {
    pub use crate::input::{DependencyGraphInput, PatternInstanceInput, ThresholdsInput};
    pub use crate::AnalysisError;
    pub use crate::DivergenceSummary;
    pub use crate::ExitCode;
    pub use crate::Snapshot;
}
