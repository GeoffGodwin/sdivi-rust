//! Dependency graph construction for sdi-rust.
//!
//! Consumes [`sdi_parsing::feature_record::FeatureRecord`] slices to build a
//! `petgraph`-backed directed dependency graph and compute graph metrics.
//!
//! # Quick Start
//!
//! ```rust
//! use sdi_graph::dependency_graph::build_dependency_graph;
//! use sdi_graph::metrics::compute_metrics;
//! use sdi_parsing::feature_record::FeatureRecord;
//! use std::path::PathBuf;
//!
//! let records: Vec<FeatureRecord> = vec![];
//! let dg = build_dependency_graph(&records);
//! let m = compute_metrics(&dg);
//! assert_eq!(m.node_count, 0);
//! ```

pub mod dependency_graph;
pub mod metrics;

pub use dependency_graph::{DependencyGraph, GraphError, build_dependency_graph};
pub use metrics::{GraphMetrics, compute_metrics};
