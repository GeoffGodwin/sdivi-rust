//! Atomic snapshot file writing.
//!
//! Snapshots are written as pretty-printed JSON via a tempfile-then-rename
//! sequence so that a killed process never leaves a half-written `.json` file
//! in `.sdi/snapshots/`.  The tempfile is created **in the same directory** as
//! the final target so that the rename is always an intra-filesystem operation
//! (POSIX atomic rename requires same mount point).

use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::snapshot::Snapshot;

/// Converts an ISO 8601 timestamp string to a filesystem-safe component.
///
/// Removes dashes and the trailing `Z`, preserving the `T` separator.
///
/// # Examples
///
/// ```
/// use sdi_snapshot::store::iso_to_filename_safe;
///
/// assert_eq!(iso_to_filename_safe("2026-04-29T12:34:56Z"), "20260429T123456");
/// assert_eq!(iso_to_filename_safe("2026-01-01T00:00:00Z"), "20260101T000000");
/// ```
pub fn iso_to_filename_safe(ts: &str) -> String {
    ts.chars()
        .filter(|&c| c != '-' && c != ':' && c != 'Z')
        .collect()
}

/// Writes `snapshot` atomically to `dir` and returns the path of the new file.
///
/// The write sequence is:
/// 1. Serialize the snapshot to pretty-printed JSON.
/// 2. Compute an 8-character `blake3` hex digest of the JSON bytes.
/// 3. Derive a filename of the form `snapshot_<YYYYMMDDTHHMMSS>_<hash8>.json`.
/// 4. Create `dir` if it does not already exist.
/// 5. Write the JSON to a [`tempfile::NamedTempFile`] inside `dir` (same
///    filesystem as the final target — required for atomic POSIX rename).
/// 6. Persist (rename) the tempfile to the final path.
///
/// # Errors
///
/// Returns [`std::io::Error`] if serialization, directory creation, temp-file
/// creation, writing, or the rename fails.
pub fn write_snapshot(snapshot: &Snapshot, dir: &Path) -> std::io::Result<PathBuf> {
    // Serialize to pretty JSON.
    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let bytes = json.as_bytes();

    // 8-char blake3 hex digest for a collision-resistant filename component.
    let hash_hex = blake3::hash(bytes).to_hex();
    let hash8 = &hash_hex[..8];

    let ts_safe = iso_to_filename_safe(&snapshot.timestamp);
    let filename = format!("snapshot_{}_{}.json", ts_safe, hash8);

    // Ensure the target directory exists.
    std::fs::create_dir_all(dir)?;

    let final_path = dir.join(&filename);

    // Create the tempfile in `dir` — NOT in /tmp — so the rename is atomic.
    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    tmp.write_all(bytes)?;

    // Atomic rename; extract the underlying I/O error on failure.
    tmp.persist(&final_path)
        .map_err(|e| e.error)?;

    tracing::debug!(path = %final_path.display(), "snapshot written");

    Ok(final_path)
}

#[cfg(test)]
mod tests {
    use super::iso_to_filename_safe;

    #[test]
    fn iso_roundtrip_basic() {
        assert_eq!(
            iso_to_filename_safe("2026-04-29T12:34:56Z"),
            "20260429T123456"
        );
    }

    #[test]
    fn iso_roundtrip_midnight() {
        assert_eq!(
            iso_to_filename_safe("2026-01-01T00:00:00Z"),
            "20260101T000000"
        );
    }

    #[test]
    fn iso_no_z_suffix_passthrough() {
        // Timestamps that omit Z are left intact aside from dash/colon removal.
        assert_eq!(
            iso_to_filename_safe("2026-04-29T12:34:56"),
            "20260429T123456"
        );
    }
}
