//! # sdivi-pipeline
//!
//! Orchestration crate for the Structural Divergence Indexer.
//!
//! Owns all FS I/O, clock access, and atomic writes.  The pure-compute facade
//! (WASM-compatible, no FS) lives in `sdivi-core`.
//!
//! # Quick start
//!
//! ```rust
//! use sdivi_pipeline::Pipeline;
//! use sdivi_config::Config;
//!
//! let pipeline = Pipeline::new(Config::default(), vec![]);
//! // Call pipeline.snapshot(repo_root, commit, timestamp) to run all five stages.
//! ```

pub mod boundaries;
pub mod cache;
pub mod change_coupling;
pub mod commit_extract;
pub mod error;
mod helpers;
pub mod pipeline;
pub mod store;
pub mod time;

pub use boundaries::{infer_from_snapshots, read_prior_partitions};
pub use change_coupling::{collect_cochange_events, ChangeCouplingError};
pub use commit_extract::CommitExtractError;
pub use error::PipelineError;
pub use pipeline::{Pipeline, WriteMode};
pub use store::{latest_snapshot, read_snapshot_by_id, read_snapshots, write_boundary_spec};
pub use time::current_timestamp;
