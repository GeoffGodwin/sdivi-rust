//! Native Leiden community detection for sdi-rust.
//!
//! Provides a fully deterministic, native Rust port of the Leiden algorithm
//! (Traag et al. 2019) with Modularity and CPM quality functions.
//!
//! # Quick Start
//!
//! ```rust
//! use sdi_detection::leiden::run_leiden;
//! use sdi_detection::partition::{LeidenConfig, LeidenPartition};
//! use sdi_graph::dependency_graph::build_dependency_graph_from_edges;
//!
//! let dg = build_dependency_graph_from_edges(&[], &[]);
//! let cfg = LeidenConfig::default();
//! let partition = run_leiden(&dg, &cfg, None);
//! assert_eq!(partition.community_count(), 0);
//! ```

pub mod leiden;
pub mod partition;
pub mod warm_start;

pub use partition::{LeidenConfig, LeidenPartition, QualityFunction};
pub use leiden::run_leiden;
pub use warm_start::{initial_assignment_from_cache, CACHE_FILENAME};
