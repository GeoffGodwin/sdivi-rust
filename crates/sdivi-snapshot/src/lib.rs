//! Snapshot assembly, delta computation, trend, and boundary inference for sdivi-rust.
//!
//! Core module for M07/M08. Assembles pipeline stage outputs into a versioned
//! `Snapshot` JSON (`snapshot_version: "1.0"`) and writes it atomically
//! (tempfile + rename) to `.sdivi/snapshots/` (requires `pipeline-records` feature).

pub mod boundary_inference;
pub mod change_coupling;
pub mod delta;
pub mod retention;
pub mod snapshot;
pub mod store;
pub mod trend;

pub use boundary_inference::{infer_boundaries, BoundaryInferenceResult, PriorPartition};
pub use change_coupling::{ChangeCouplingResult, CoChangePair};
pub use delta::{compute_delta, null_summary, DivergenceSummary};
pub use snapshot::{
    assemble_snapshot, IntentDivergenceInfo, PatternMetricsResult, Snapshot, SNAPSHOT_VERSION,
};
pub use trend::{compute_trend, TrendResult};

#[cfg(feature = "pipeline-records")]
pub use retention::enforce_retention;
#[cfg(feature = "pipeline-records")]
pub use store::write_snapshot;
