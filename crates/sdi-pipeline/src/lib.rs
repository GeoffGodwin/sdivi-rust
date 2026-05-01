//! # sdi-pipeline
//!
//! Orchestration crate for the Structural Divergence Indexer.
//!
//! Owns all FS I/O, clock access, and atomic writes.  The pure-compute facade
//! (WASM-compatible, no FS) lives in `sdi-core`.
//!
//! # Quick start
//!
//! ```rust
//! use sdi_pipeline::Pipeline;
//! use sdi_config::Config;
//!
//! let pipeline = Pipeline::new(Config::default(), vec![]);
//! // Call pipeline.snapshot(repo_root, commit, timestamp) to run all five stages.
//! ```

pub mod boundaries;
pub mod cache;
pub mod error;
pub mod pipeline;
pub mod store;

pub use boundaries::{infer_from_snapshots, read_prior_partitions};
pub use error::PipelineError;
pub use pipeline::{Pipeline, WriteMode, current_timestamp};
pub use store::{latest_snapshot, read_snapshot_by_id, read_snapshots, write_boundary_spec};
