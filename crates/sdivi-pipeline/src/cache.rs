//! Partition cache I/O — load and save the warm-start `LeidenPartition`.
//!
//! Pure logic (initial assignment mapping) lives in `sdivi_detection::warm_start`.
//! This module owns the FS operations only.

use std::io::Write as _;
use std::path::Path;

use sdivi_detection::partition::LeidenPartition;
use tracing::debug;

/// Loads a prior partition from `cache_path` if it exists.
///
/// Returns `None` if the file is absent or cannot be parsed (both conditions
/// are logged at `DEBUG` level and treated as a cold-start).
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use sdivi_pipeline::cache::load_cached_partition;
///
/// let result = load_cached_partition(Path::new("/nonexistent/partition.json"));
/// assert!(result.is_none());
/// ```
pub fn load_cached_partition(cache_path: &Path) -> Option<LeidenPartition> {
    match std::fs::read_to_string(cache_path) {
        Ok(json) => match LeidenPartition::from_json(&json) {
            Ok(p) => {
                debug!(path = %cache_path.display(), "warm-start partition loaded");
                Some(p)
            }
            Err(e) => {
                debug!(path = %cache_path.display(), error = %e, "partition parse error — cold start");
                None
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!(path = %cache_path.display(), "no cached partition — cold start");
            None
        }
        Err(e) => {
            debug!(path = %cache_path.display(), error = %e, "partition read error — cold start");
            None
        }
    }
}

/// Saves a [`LeidenPartition`] to `cache_path` atomically.
///
/// The parent directory is created if it does not exist.
pub fn save_cached_partition(
    partition: &LeidenPartition,
    cache_path: &Path,
) -> std::io::Result<()> {
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = partition
        .to_json()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let dir = cache_path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    tmp.write_all(json.as_bytes())?;
    tmp.persist(cache_path).map_err(|e| e.error)?;

    debug!(path = %cache_path.display(), "cached partition saved");
    Ok(())
}
