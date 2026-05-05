//! Dependency graph construction for sdivi-rust.
//!
//! Builds a `petgraph`-backed directed dependency graph.  Two entry points:
//! - [`build_dependency_graph`] (feature `pipeline-records`, ON by default) — takes
//!   [`sdivi_parsing::feature_record::FeatureRecord`] slices from the parsing stage.
//! - [`build_dependency_graph_from_edges`] (always available) — takes raw node paths
//!   and `(from, to)` edge pairs for WASM / pure-compute consumers.
//!
//! # Quick Start
//!
//! ```rust
//! use sdivi_graph::dependency_graph::build_dependency_graph_from_edges;
//! use sdivi_graph::metrics::compute_metrics;
//!
//! let dg = build_dependency_graph_from_edges(&[], &[]);
//! let m = compute_metrics(&dg);
//! assert_eq!(m.node_count, 0);
//! ```

pub mod dependency_graph;
pub mod metrics;
pub(crate) mod resolve;
pub(crate) mod resolve_lang;

pub use dependency_graph::{build_dependency_graph_from_edges, DependencyGraph, GraphError};
pub use metrics::{compute_metrics, GraphMetrics};

#[cfg(feature = "pipeline-records")]
pub use dependency_graph::{build_dependency_graph, build_dependency_graph_with_go_module};
