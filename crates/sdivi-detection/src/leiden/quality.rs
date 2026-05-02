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
pub(crate) fn compute_modularity(graph: &LeidenGraph, assignment: &[usize]) -> f64 {
    let m = graph.total_weight;
    if m == 0.0 {
        return 0.0;
    }
    let max_comm = assignment.iter().copied().max().map(|m| m + 1).unwrap_or(0);
    let mut sigma = vec![0.0f64; max_comm];
    let mut inner = vec![0.0f64; max_comm];

    for (i, &c) in assignment.iter().enumerate() {
        sigma[c] += graph.degree[i];
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
