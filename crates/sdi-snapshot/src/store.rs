//! Atomic snapshot file writing — only with `pipeline-records` feature.
//!
//! The actual orchestration write path lives in `sdi-pipeline::store` which
//! re-exports these helpers.  With `pipeline-records` feature off (WASM build),
//! this module is absent from the compilation unit.

#![cfg(feature = "pipeline-records")]

use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::snapshot::Snapshot;

/// Converts an ISO 8601 timestamp string to a filesystem-safe component.
///
/// # Examples
///
/// ```
/// use sdi_snapshot::store::iso_to_filename_safe;
///
/// assert_eq!(iso_to_filename_safe("2026-04-29T12:34:56Z"), "20260429T123456");
/// ```
pub fn iso_to_filename_safe(ts: &str) -> String {
    ts.chars()
        .filter(|&c| c != '-' && c != ':' && c != 'Z')
        .collect()
}

/// Writes `snapshot` atomically to `dir` and returns the path of the new file.
pub fn write_snapshot(snapshot: &Snapshot, dir: &Path) -> std::io::Result<PathBuf> {
    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let bytes = json.as_bytes();
    let hash_hex = blake3::hash(bytes).to_hex();
    let hash8 = &hash_hex[..8];
    let ts_safe = iso_to_filename_safe(&snapshot.timestamp);
    let filename = format!("snapshot_{}_{}.json", ts_safe, hash8);

    std::fs::create_dir_all(dir)?;
    let final_path = dir.join(&filename);

    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    tmp.write_all(bytes)?;
    tmp.persist(&final_path).map_err(|e| e.error)?;

    tracing::debug!(path = %final_path.display(), "snapshot written");
    Ok(final_path)
}

#[cfg(test)]
mod tests {
    use super::iso_to_filename_safe;

    #[test]
    fn iso_roundtrip_basic() {
        assert_eq!(iso_to_filename_safe("2026-04-29T12:34:56Z"), "20260429T123456");
    }

    #[test]
    fn iso_roundtrip_midnight() {
        assert_eq!(iso_to_filename_safe("2026-01-01T00:00:00Z"), "20260101T000000");
    }
}
