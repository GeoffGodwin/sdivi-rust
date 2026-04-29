#![deny(missing_docs)]
//! # sdi-core
//!
//! Core library for the Structural Divergence Indexer (sdi-rust).
//!
//! This crate is the stable public surface for embedding the analysis pipeline
//! in Rust programs, PyO3 wheels, and napi-rs bindings. Every feature exposed
//! by `sdi-cli` is reachable through this crate.
//!
//! # Quick start
//!
//! ```rust
//! use sdi_core::ExitCode;
//!
//! assert_eq!(ExitCode::Success as i32, 0);
//! ```

/// Errors produced by the sdi-core analysis pipeline.
pub mod error;

/// Exit codes for the `sdi` binary — public API, adding variants is a breaking change.
pub mod exit_code;

/// Five-stage analysis pipeline — [`Pipeline::snapshot`] and [`Pipeline::delta`].
pub mod pipeline;

pub use error::AnalysisError;
pub use exit_code::ExitCode;
pub use pipeline::Pipeline;

/// Pattern fingerprinting and catalog — re-exported from `sdi-patterns`.
pub use sdi_patterns::{
    build_catalog, compute_entropy, fingerprint_node_kind, PatternCatalog, PatternFingerprint,
    PatternLocation, PatternStats, FINGERPRINT_KEY,
};

/// Snapshot types and functions — re-exported from `sdi-snapshot`.
pub use sdi_snapshot::{
    build_snapshot, compute_delta, enforce_retention, null_summary, write_snapshot,
    DivergenceSummary, IntentDivergenceInfo, Snapshot, SNAPSHOT_VERSION,
};

/// Commonly-imported items from sdi-core.
pub mod prelude {
    pub use crate::error::AnalysisError;
    pub use crate::exit_code::ExitCode;
    pub use crate::pipeline::Pipeline;
}
