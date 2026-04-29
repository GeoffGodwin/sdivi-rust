//! Synchronous retention enforcement for the snapshot directory.
//!
//! After each successful atomic write, call [`enforce_retention`] to remove
//! the oldest snapshots beyond the configured maximum.  Retention is
//! **synchronous** — it runs in the same thread as the write so the
//! directory count is always consistent after a successful snapshot operation.
//!
//! A `max` of `0` means unlimited; the function returns immediately.

use std::path::{Path, PathBuf};

/// Enforces a maximum number of snapshot files in `dir`.
///
/// Files matching the pattern `snapshot_*.json` are considered.  They are
/// sorted lexicographically — the filename format `snapshot_YYYYMMDDTHHMMSS_*`
/// guarantees that lexicographic order equals chronological order.  When the
/// count exceeds `max`, the oldest `count - max` files are deleted.
///
/// Passing `max = 0` disables retention (unlimited snapshots).
///
/// # Errors
///
/// Returns [`std::io::Error`] if the directory cannot be read or a file cannot
/// be removed.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use sdi_snapshot::retention::enforce_retention;
///
/// // Keep at most 100 snapshots in `.sdi/snapshots`.
/// enforce_retention(Path::new(".sdi/snapshots"), 100)?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn enforce_retention(dir: &Path, max: u32) -> std::io::Result<()> {
    if max == 0 {
        return Ok(());
    }

    let mut candidates: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("snapshot_") && name_str.ends_with(".json") {
                Some(entry.path())
            } else {
                None
            }
        })
        .collect();

    // Lexicographic sort → chronological order (oldest first).
    candidates.sort();

    let count = candidates.len();
    let max_usize = max as usize;

    if count > max_usize {
        let to_delete = count - max_usize;
        for path in candidates.iter().take(to_delete) {
            tracing::debug!(path = %path.display(), "removing old snapshot (retention)");
            std::fs::remove_file(path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::enforce_retention;
    use std::fs;
    use tempfile::TempDir;

    fn touch(dir: &std::path::Path, name: &str) {
        fs::write(dir.join(name), b"{}").unwrap();
    }

    #[test]
    fn zero_max_is_unlimited() {
        let dir = TempDir::new().unwrap();
        for i in 0..10u32 {
            touch(dir.path(), &format!("snapshot_2026010{i}T000000_abcd1234.json"));
        }
        enforce_retention(dir.path(), 0).unwrap();
        let remaining = fs::read_dir(dir.path()).unwrap().count();
        assert_eq!(remaining, 10);
    }

    #[test]
    fn keeps_newest_files() {
        let dir = TempDir::new().unwrap();
        // Create 5 snapshots in chronological order.
        for i in 1..=5u32 {
            touch(
                dir.path(),
                &format!("snapshot_2026010{i}T000000_abcd1234.json"),
            );
        }
        enforce_retention(dir.path(), 3).unwrap();
        let mut remaining: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        remaining.sort();
        assert_eq!(remaining.len(), 3);
        // The three newest (days 3, 4, 5) should remain.
        assert!(remaining[0].contains("20260103"));
        assert!(remaining[1].contains("20260104"));
        assert!(remaining[2].contains("20260105"));
    }

    #[test]
    fn non_snapshot_files_ignored() {
        let dir = TempDir::new().unwrap();
        touch(dir.path(), "README.txt");
        touch(dir.path(), "snapshot_20260101T000000_aabbccdd.json");
        touch(dir.path(), "snapshot_20260102T000000_11223344.json");
        enforce_retention(dir.path(), 1).unwrap();
        // Only snapshot files count; README.txt should be untouched.
        assert!(dir.path().join("README.txt").exists());
        // One snapshot should remain — the newest.
        let snapshots: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| {
                let e = e.unwrap();
                let n = e.file_name().into_string().unwrap();
                if n.starts_with("snapshot_") { Some(n) } else { None }
            })
            .collect();
        assert_eq!(snapshots.len(), 1);
        assert!(snapshots[0].contains("20260102"));
    }
}
