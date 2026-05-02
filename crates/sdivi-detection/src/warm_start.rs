//! Warm-start support for the Leiden algorithm.
//!
//! The partition cache allows consecutive snapshot runs to start from the
//! previous community assignments rather than all-singletons, improving
//! stability across incremental changes.  FS I/O (load / save) lives in
//! `sdivi-pipeline::cache`; this module contains only the pure mapping logic.

use std::collections::BTreeMap;

use crate::partition::LeidenPartition;

/// Default cache path relative to the `.sdivi` directory.
///
/// The FS operations using this constant live in `sdivi-pipeline::cache`.
pub const CACHE_FILENAME: &str = "cache/partition.json";

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
/// use sdivi_detection::partition::LeidenPartition;
/// use sdivi_detection::warm_start::initial_assignment_from_cache;
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
    let mut assignment: Vec<usize> = (0..node_count).collect();

    let cached = match cached {
        Some(c) => c,
        None => return assignment,
    };

    let mut community_remap: BTreeMap<usize, usize> = BTreeMap::new();
    let mut next_comm = 0usize;

    for (&node, &comm) in &cached.assignments {
        if node >= node_count {
            continue;
        }
        let remapped = *community_remap.entry(comm).or_insert_with(|| {
            let c = next_comm;
            next_comm += 1;
            c
        });
        assignment[node] = remapped;
    }

    let mut used: std::collections::BTreeSet<usize> = community_remap.values().copied().collect();
    let mut counter = 0usize;

    for (i, slot) in assignment.iter_mut().enumerate().take(node_count) {
        if !cached.assignments.contains_key(&i) {
            while used.contains(&counter) {
                counter += 1;
            }
            *slot = counter;
            used.insert(counter);
            counter += 1;
        }
    }

    assignment
}
