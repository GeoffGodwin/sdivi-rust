#![deny(missing_docs)]
//! Pattern fingerprinting and catalog for sdivi-rust.
//!
//! Implements Stage 4 of the five-stage analysis pipeline. Classifies
//! pattern hints into the five built-in categories, fingerprints their
//! structural shapes with `blake3`, and assembles a [`PatternCatalog`] with
//! per-category entropy.
//!
//! # Design constraints
//!
//! This crate must NOT depend on `sdivi-graph` or `sdivi-detection`.
//!
//! # Quick start
//!
//! ```rust
//! use sdivi_patterns::PatternCatalog;
//!
//! let catalog = PatternCatalog::default();
//! assert!(catalog.entries.is_empty());
//! ```

pub mod catalog;
pub mod entropy;
pub mod fingerprint;
pub mod normalize;
pub mod queries;

pub use catalog::{PatternCatalog, PatternLocation, PatternStats};
pub use entropy::compute_entropy;
pub use fingerprint::{fingerprint_node_kind, PatternFingerprint, FINGERPRINT_KEY};
pub use normalize::{normalize_and_hash, NormalizeNode};

#[cfg(feature = "pipeline-records")]
pub use catalog::build_catalog;
