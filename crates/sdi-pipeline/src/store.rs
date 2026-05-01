//! Snapshot persistence helpers — atomic write, retention, and read utilities.
//!
//! Re-exports the write helpers from `sdi-snapshot`; the read utilities
//! (listing snapshots, loading by id, loading the latest) live here.

use std::path::Path;

pub use sdi_snapshot::store::{iso_to_filename_safe, write_snapshot};
pub use sdi_snapshot::retention::enforce_retention;

use sdi_snapshot::Snapshot;

/// Reads all snapshots from `dir` in chronological order (oldest→newest).
///
/// Files are sorted lexicographically by filename. The `snapshot_*` naming
/// scheme (`snapshot_<YYYYMMDDTHHMMSS>_<hash>.json`) ensures this matches
/// chronological order. Non-matching files and malformed JSON are skipped
/// with a warning logged to stderr.
///
/// Returns an empty `Vec` if the directory does not exist.
pub fn read_snapshots(dir: &Path) -> std::io::Result<Vec<Snapshot>> {
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let s = name.to_string_lossy();
            s.starts_with("snapshot_") && s.ends_with(".json")
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());

    let mut snapshots = Vec::new();
    for entry in entries {
        let content = std::fs::read_to_string(entry.path())?;
        match serde_json::from_str::<Snapshot>(&content) {
            Ok(s) => snapshots.push(s),
            Err(e) => {
                tracing::warn!(
                    path = %entry.path().display(),
                    error = %e,
                    "skipping malformed snapshot"
                );
            }
        }
    }
    Ok(snapshots)
}

/// Returns the most recent snapshot from `dir`, or `None` if none exist.
///
/// "Most recent" is the lexicographically last `snapshot_*.json` file,
/// which matches chronological order under the M07 naming scheme.
pub fn latest_snapshot(dir: &Path) -> std::io::Result<Option<Snapshot>> {
    if !dir.exists() {
        return Ok(None);
    }
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let s = name.to_string_lossy();
            s.starts_with("snapshot_") && s.ends_with(".json")
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());

    match entries.last() {
        None => Ok(None),
        Some(entry) => {
            let content = std::fs::read_to_string(entry.path())?;
            let snap = serde_json::from_str(&content)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            Ok(Some(snap))
        }
    }
}

/// Returns the snapshot identified by `id` from `dir`.
///
/// `id` is matched as the filename stem (without `.json` extension).
/// For example, `"snapshot_20260429T123456_abc12345"` matches the file
/// `snapshot_20260429T123456_abc12345.json` in `dir`.
pub fn read_snapshot_by_id(dir: &Path, id: &str) -> std::io::Result<Snapshot> {
    let filename = if id.ends_with(".json") {
        id.to_string()
    } else {
        format!("{id}.json")
    };
    let path = dir.join(&filename);
    let content = std::fs::read_to_string(&path)
        .map_err(|e| {
            std::io::Error::new(e.kind(), format!("snapshot '{}' not found: {e}", id))
        })?;
    serde_json::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}
