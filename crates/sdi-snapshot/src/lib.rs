//! Snapshot assembly, delta computation, and atomic persistence for sdi-rust.
//!
//! Implemented in Milestone 7. Assembles pipeline stage outputs into a
//! versioned `Snapshot` JSON (`snapshot_version: "1.0"`) and writes it
//! atomically (tempfile + rename) to `.sdi/snapshots/`.

pub mod delta;
pub mod retention;
pub mod snapshot;
pub mod store;

pub use delta::{compute_delta, null_summary, DivergenceSummary};
pub use retention::enforce_retention;
pub use snapshot::{build_snapshot, IntentDivergenceInfo, Snapshot, SNAPSHOT_VERSION};
pub use store::write_snapshot;
