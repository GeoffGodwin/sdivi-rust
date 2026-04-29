//! Warm-start support: load a prior [`LeidenPartition`] from the snapshot cache.
//!
//! When `.sdi/cache/partition.json` exists and its node set overlaps the
//! current graph's node set, the Leiden run is initialised from those community
//! assignments rather than starting fresh (one node per community).  Nodes
//! absent from the cached partition fall back to singleton communities.

use std::collections::BTreeMap;
use std::path::Path;

use tracing::debug;

use crate::partition::LeidenPartition;

/// Default cache path relative to the `.sdi` directory.
pub const CACHE_FILENAME: &str = "cache/partition.json";

/// Loads a prior partition from `cache_path` if it exists.
///
/// Returns `None` if the file is absent or cannot be parsed (both conditions
/// are logged at `DEBUG` level and treated as a cold-start).
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use sdi_detection::warm_start::load_cached_partition;
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

/// Converts a cached partition into an initial assignment vector.
///
/// `node_count` is the number of nodes in the current graph.
/// Nodes not present in `cached` fall back to singleton assignments
/// (`assignment[i] = i`).
///
/// Community IDs from the cache are remapped so they stay in `[0, node_count)`.
///
/// # Examples
///
/// ```rust
/// use std::collections::BTreeMap;
/// use sdi_detection::partition::LeidenPartition;
/// use sdi_detection::warm_start::initial_assignment_from_cache;
///
/// let cached = LeidenPartition {
///     assignments: BTreeMap::from([(0, 0), (1, 0)]),
///     stability: BTreeMap::from([(0, 0.9)]),
///     modularity: 0.5,
///     seed: 42,
/// };
/// let init = initial_assignment_from_cache(Some(&cached), 3);
/// assert_eq!(init[0], init[1]); // same community
/// assert_ne!(init[0], init[2]); // node 2 falls back to singleton
/// ```
pub fn initial_assignment_from_cache(
    cached: Option<&LeidenPartition>,
    node_count: usize,
) -> Vec<usize> {
    // Default: each node is its own community.
    let mut assignment: Vec<usize> = (0..node_count).collect();

    let cached = match cached {
        Some(c) => c,
        None => return assignment,
    };

    // Remap cached community IDs to guarantee they stay in-range.
    let mut community_remap: BTreeMap<usize, usize> = BTreeMap::new();
    let mut next_comm = 0usize;

    for (&node, &comm) in &cached.assignments {
        if node >= node_count {
            continue; // node no longer exists — skip
        }
        let remapped = *community_remap.entry(comm).or_insert_with(|| {
            let c = next_comm;
            next_comm += 1;
            c
        });
        assignment[node] = remapped;
    }

    // Nodes without a cache entry keep their singleton community.
    // Renumber singletons to avoid colliding with remapped IDs.
    let mut used: std::collections::BTreeSet<usize> =
        community_remap.values().copied().collect();
    let mut counter = 0usize;

    for i in 0..node_count {
        if !cached.assignments.contains_key(&i) {
            // Find next unused community ID.
            while used.contains(&counter) {
                counter += 1;
            }
            assignment[i] = counter;
            used.insert(counter);
            counter += 1;
        }
    }

    assignment
}

/// Saves a [`LeidenPartition`] to `cache_path` atomically.
///
/// The parent directory is created if it does not exist. Writes to a temp file
/// in the same directory and renames — a killed process will not leave a
/// half-written file.
pub fn save_cached_partition(
    partition: &LeidenPartition,
    cache_path: &Path,
) -> std::io::Result<()> {
    use std::io::Write as _;

    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = partition.to_json().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    })?;

    let dir = cache_path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    tmp.write_all(json.as_bytes())?;
    tmp.persist(cache_path).map_err(|e| e.error)?;

    debug!(path = %cache_path.display(), "cached partition saved");
    Ok(())
}
