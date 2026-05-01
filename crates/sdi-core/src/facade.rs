//! Re-exports of snapshot-layer functions for convenience.
//!
//! These are the "assembly-and-delta" functions that sdi-core surfaces from
//! `sdi-snapshot`.  Embedders that only need the pure-compute functions can
//! call `crate::compute::*` directly; embedders that need to assemble or
//! diff snapshots use these re-exports.

/// Assemble a [`crate::Snapshot`] from pipeline stage outputs.
///
/// See [`sdi_snapshot::snapshot::assemble_snapshot`] for full documentation.
pub use sdi_snapshot::snapshot::assemble_snapshot;

/// Compute per-dimension divergence between two snapshots.
///
/// See [`sdi_snapshot::delta::compute_delta`] for full documentation.
pub use sdi_snapshot::delta::compute_delta;

/// Return a [`crate::DivergenceSummary`] with all fields `None` (first-snapshot path).
///
/// See [`sdi_snapshot::delta::null_summary`] for full documentation.
pub use sdi_snapshot::delta::null_summary;

/// Compute trend statistics over a slice of snapshots.
///
/// See [`sdi_snapshot::trend::compute_trend`] for full documentation.
pub use sdi_snapshot::trend::compute_trend;

/// Infer boundary proposals from a sequence of prior partitions.
///
/// See [`sdi_snapshot::boundary_inference::infer_boundaries`] for full documentation.
pub use sdi_snapshot::boundary_inference::infer_boundaries;
