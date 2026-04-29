//! sdi-rust: Structural Divergence Indexer — install-discovery meta-crate.
//!
//! This crate exists for name discoverability on crates.io. To install the
//! `sdi` binary, run:
//!
//! ```text
//! cargo install sdi-cli
//! ```
//!
//! To embed the analysis pipeline in your Rust project, depend on `sdi-core`:
//!
//! ```text
//! [dependencies]
//! sdi-core = "*"
//! ```

pub use sdi_core as core;
