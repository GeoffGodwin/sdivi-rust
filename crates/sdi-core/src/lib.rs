#![deny(missing_docs)]
//! # sdi-core
//!
//! Pure-compute facade for the Structural Divergence Indexer (sdi-rust).
//!
//! This crate is the stable, WASM-compatible public surface for embedding the
//! analysis pipeline in Rust programs, WASM bindings, and language bindings.
//! It has **no I/O, no clock, no tree-sitter, and no `std::fs`** — callers
//! supply pre-extracted data via the `*Input` struct family and receive
//! plain `serde` result types.
//!
//! Embedders that need the full FS-orchestrated pipeline (parsing, snapshot
//! writes, retention) should use `sdi-pipeline` instead.
//!
//! # Quick start
//!
//! ```rust
//! use sdi_core::ExitCode;
//!
//! assert_eq!(ExitCode::Success as i32, 0);
//! ```
//!
//! ## Pure-compute example
//!
//! ```rust
//! use sdi_core::compute::coupling::compute_coupling_topology;
//! use sdi_core::input::DependencyGraphInput;
//!
//! let g = DependencyGraphInput { nodes: vec![], edges: vec![] };
//! let result = compute_coupling_topology(&g).unwrap();
//! assert_eq!(result.node_count, 0);
//! ```

/// Errors produced by the sdi-core pure-compute API.
pub mod error;

/// Exit codes for the `sdi` binary — public API, adding variants is a breaking change.
pub mod exit_code;

/// Input structs for the pure-compute API (WASM-safe serde types).
pub mod input;

/// Pure-compute functions over [`input`] structs.
pub mod compute;

/// Re-exports of snapshot assembly, delta, trend, and boundary inference from `sdi-snapshot`.
pub mod facade;

pub use error::AnalysisError;
pub use exit_code::ExitCode;

// ── input type re-exports ──────────────────────────────────────────────────

pub use input::{
    BoundaryDefInput, BoundarySpecInput, ChangeCouplingConfigInput, CoChangeEventInput,
    DependencyGraphInput, EdgeInput, LeidenConfigInput,
    NodeInput, NormalizeNode, PatternInstanceInput, PatternLocationInput, PriorPartition,
    QualityFunctionInput, ThresholdOverrideInput, ThresholdsInput, validate_node_id,
    edge_weight_key, split_edge_weight_key,
};

// ── compute function re-exports ────────────────────────────────────────────

pub use compute::boundaries::{
    BoundaryDetectionResult, BoundaryViolationResult, compute_boundary_violations, detect_boundaries,
};
pub use compute::change_coupling::compute_change_coupling;
pub use compute::coupling::{CouplingTopologyResult, compute_coupling_topology};
pub use compute::normalize::normalize_and_hash;
pub use compute::patterns::{compute_pattern_metrics, compute_pattern_metrics_from_catalog};
pub use compute::thresholds::{
    AppliedOverrideInfo, ThresholdBreachInfo, ThresholdCheckResult, compute_thresholds_check,
};

// ── facade re-exports (sdi-snapshot) ──────────────────────────────────────

pub use facade::{assemble_snapshot, compute_delta, compute_trend, infer_boundaries, null_summary};

// ── snapshot types re-exported directly ───────────────────────────────────

pub use sdi_snapshot::boundary_inference::{
    BoundaryInferenceResult, BoundaryProposal,
    PriorPartition as SnapshotPriorPartition,
};
pub use sdi_snapshot::change_coupling::{ChangeCouplingResult, CoChangePair};
pub use sdi_snapshot::delta::DivergenceSummary;
pub use sdi_snapshot::snapshot::{
    IntentDivergenceInfo, PatternMetricsResult, Snapshot, SNAPSHOT_VERSION,
};
pub use sdi_snapshot::trend::TrendResult;

// ── fingerprint key re-export ──────────────────────────────────────────────

/// The fixed `blake3` key used for all pattern fingerprints.
///
/// Foreign extractors that produce pattern fingerprints must use this same key
/// to ensure their fingerprints are byte-identical to those produced by the
/// native Rust pipeline.
///
/// See [`normalize_and_hash`] for the canonical tree-aware algorithm.
pub use sdi_patterns::FINGERPRINT_KEY;

// ── inner-crate type re-exports (for sdi-wasm and other embedders) ─────────

/// Graph metrics summary — re-exported from `sdi-graph` for WASM embedders.
pub use sdi_graph::metrics::GraphMetrics;

/// Leiden community detection result — re-exported from `sdi-detection` for WASM embedders.
pub use sdi_detection::partition::LeidenPartition;

/// Pattern fingerprint catalog — re-exported from `sdi-patterns` for WASM embedders.
pub use sdi_patterns::catalog::{PatternCatalog, PatternStats};

/// Pattern fingerprint type — re-exported from `sdi-patterns` for WASM embedders.
pub use sdi_patterns::fingerprint::PatternFingerprint;

/// Commonly-imported items from sdi-core.
pub mod prelude {
    pub use crate::AnalysisError;
    pub use crate::ExitCode;
    pub use crate::DivergenceSummary;
    pub use crate::Snapshot;
    pub use crate::input::{DependencyGraphInput, PatternInstanceInput, ThresholdsInput};
}
