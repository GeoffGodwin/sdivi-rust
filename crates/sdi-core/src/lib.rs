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

pub use error::AnalysisError;
pub use exit_code::ExitCode;

/// Pattern fingerprinting and catalog — re-exported from `sdi-patterns`.
pub use sdi_patterns::{
    build_catalog, compute_entropy, fingerprint_node_kind, PatternCatalog, PatternFingerprint,
    PatternLocation, PatternStats, FINGERPRINT_KEY,
};

/// Commonly-imported items from sdi-core.
pub mod prelude {
    pub use crate::error::AnalysisError;
    pub use crate::exit_code::ExitCode;
}
