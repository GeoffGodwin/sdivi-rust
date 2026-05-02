//! Native Leiden community detection for sdivi-rust.
//!
//! Provides a fully deterministic, native Rust port of the Leiden algorithm
//! (Traag et al. 2019) with Modularity and CPM quality functions.
//!
//! # Quick Start
//!
//! ```rust
//! use sdivi_detection::leiden::run_leiden;
//! use sdivi_detection::partition::{LeidenConfig, LeidenPartition};
//! use sdivi_graph::dependency_graph::build_dependency_graph_from_edges;
//!
//! let dg = build_dependency_graph_from_edges(&[], &[]);
//! let cfg = LeidenConfig::default();
//! let partition = run_leiden(&dg, &cfg, None);
//! assert_eq!(partition.community_count(), 0);
//! ```

pub mod leiden;
pub mod partition;
pub mod warm_start;

pub use leiden::run_leiden;
pub use leiden::run_leiden_with_weights;
pub use partition::{LeidenConfig, LeidenPartition, QualityFunction};
pub use warm_start::{initial_assignment_from_cache, CACHE_FILENAME};

/// Internal test helpers — **not stable API**.
///
/// These re-exports exist solely for integration-test plumbing (e.g.
/// `tests/aggregate_invariance.rs`).  They may be removed or renamed at any
/// time; do not use them in production code.
#[doc(hidden)]
pub mod internal {
    pub use crate::leiden::aggregate::{aggregate_network, AggregateResult};
    pub use crate::leiden::graph::LeidenGraph;
    pub use crate::leiden::quality::compute_modularity;
    pub use crate::leiden::refine::{refine_partition, well_connected, RefinementState};
}
