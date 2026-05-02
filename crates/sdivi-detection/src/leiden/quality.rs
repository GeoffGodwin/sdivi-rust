//! Per-community stability (internal edge density) computation.

use super::graph::LeidenGraph;
use std::collections::BTreeMap;

/// Computes per-community stability (internal edge density — weighted).
pub(crate) fn compute_stability(graph: &LeidenGraph, assignment: &[usize]) -> BTreeMap<usize, f64> {
    let max_comm = assignment.iter().copied().max().map(|m| m + 1).unwrap_or(0);
    let mut size = vec![0usize; max_comm];
    let mut inner = vec![0.0f64; max_comm];

    for (i, &c) in assignment.iter().enumerate() {
        size[c] += 1;
        // Self-loops are internal to node i's community.
        inner[c] += graph.self_loops[i];
        for (idx, &j) in graph.adj[i].iter().enumerate() {
            if assignment[j] == c && j > i {
                inner[c] += graph.edge_weights[i][idx];
            }
        }
    }

    let mut stability = BTreeMap::new();
    for c in 0..max_comm {
        if size[c] == 0 {
            continue;
        }
        let n = size[c] as f64;
        let max_possible = n * (n - 1.0) / 2.0;
        // max_possible counts only non-self-loop pairs; self-loop weight in
        // `inner[c]` is not bounded by it, so stability > 1.0 is theoretically
        // possible on graphs with self-loops.  In practice this never occurs
        // here because `build_partition` always calls `compute_stability` on a
        // `LeidenGraph` built from a `DependencyGraph`, which has no self-loops
        // (`self_loops[i] == 0.0` for every node).
        let s = if max_possible > 0.0 {
            inner[c] / max_possible
        } else {
            1.0
        };
        stability.insert(c, s);
    }
    stability
}

/// Computes overall modularity Q (weighted).
///
/// Self-loop weight on node `i` contributes to `L_c` (internal edges of community
/// `c = assignment[i]`) since a self-loop is always internal to its node's community.
///
/// # Examples
///
/// ```
/// # use sdivi_detection::internal::{LeidenGraph, compute_modularity};
/// // 2-node graph: cross-edge (0,1) weight 1.0, self-loops on both weight 0.5.
/// let g = LeidenGraph::from_edges_weighted(2, &[(0,0,0.5),(1,1,0.5),(0,1,1.0)]);
/// // Both in same community: Q = 0.
/// assert!((compute_modularity(&g, &[0, 0])).abs() < 1e-9);
/// // Each alone: Q = 0 (self-loops dominate, no community structure).
/// assert!((compute_modularity(&g, &[0, 1])).abs() < 1e-9);
/// ```
#[doc(hidden)]
pub fn compute_modularity(graph: &LeidenGraph, assignment: &[usize]) -> f64 {
    let m = graph.total_weight;
    if m == 0.0 {
        return 0.0;
    }
    let max_comm = assignment.iter().copied().max().map(|m| m + 1).unwrap_or(0);
    let mut sigma = vec![0.0f64; max_comm];
    let mut inner = vec![0.0f64; max_comm];

    for (i, &c) in assignment.iter().enumerate() {
        sigma[c] += graph.degree[i];
        // Self-loops are always internal to node i's community.
        inner[c] += graph.self_loops[i];
        for (idx, &j) in graph.adj[i].iter().enumerate() {
            if assignment[j] == c && j > i {
                inner[c] += graph.edge_weights[i][idx];
            }
        }
    }

    let m2 = 2.0 * m;
    sigma
        .iter()
        .zip(inner.iter())
        .fold(0.0, |acc, (&s, &l)| acc + l / m - (s / m2).powi(2))
}
