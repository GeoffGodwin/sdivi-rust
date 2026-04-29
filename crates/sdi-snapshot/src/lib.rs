//! Snapshot assembly, delta computation, and atomic persistence for sdi-rust.
//!
//! Implemented in Milestone 7. Assembles pipeline stage outputs into a
//! versioned `Snapshot` JSON (`snapshot_version: "1.0"`) and writes it
//! atomically (tempfile + rename) to `.sdi/snapshots/`.
