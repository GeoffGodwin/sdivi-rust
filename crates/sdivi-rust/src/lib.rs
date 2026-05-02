//! sdivi-rust: Structural Divergence Indexer — install-discovery meta-crate.
//!
//! This crate exists for name discoverability on crates.io. To install the
//! `sdivi` binary, run:
//!
//! ```text
//! cargo install sdivi-cli
//! ```
//!
//! To embed the analysis pipeline in your Rust project, depend on `sdivi-core`:
//!
//! ```text
//! [dependencies]
//! sdivi-core = "*"
//! ```

pub use sdivi_core as core;
