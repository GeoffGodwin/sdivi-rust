//! Snapshot persistence — atomic write and retention enforcement.
//!
//! Re-exports the helpers from `sdi-snapshot`; the orchestration logic
//! (choosing the path, enforcing retention after the write) lives here.

pub use sdi_snapshot::store::{iso_to_filename_safe, write_snapshot};
pub use sdi_snapshot::retention::enforce_retention;
