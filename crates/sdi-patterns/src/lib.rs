#![deny(missing_docs)]
//! Pattern fingerprinting and catalog for sdi-rust.
//!
//! Implements Stage 4 of the five-stage analysis pipeline. Classifies
//! [`sdi_parsing::feature_record::PatternHint`] instances into the five built-in
//! categories, fingerprints their structural shapes with `blake3`, and assembles
//! a [`PatternCatalog`] with per-category entropy.
//!
//! # Design constraints
//!
//! This crate must NOT depend on `sdi-graph` or `sdi-detection`. It operates
//! solely on the output of the parsing stage (`Vec<FeatureRecord>`).
//!
//! # Quick start
//!
//! ```rust
//! use sdi_config::Config;
//! use sdi_patterns::build_catalog;
//!
//! let catalog = build_catalog(&[], &Config::default().patterns);
//! assert!(catalog.entries.is_empty());
//! ```

pub mod catalog;
pub mod entropy;
pub mod fingerprint;
pub mod queries;

pub use catalog::{build_catalog, PatternCatalog, PatternLocation, PatternStats};
pub use entropy::compute_entropy;
pub use fingerprint::{fingerprint_node_kind, PatternFingerprint, FINGERPRINT_KEY};
