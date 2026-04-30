//! Graph metric computation for [`DependencyGraph`].

use std::path::PathBuf;

use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};

use crate::dependency_graph::DependencyGraph;

/// Summary metrics for a [`DependencyGraph`].
///
/// # Examples
///
/// ```rust
/// use sdi_graph::dependency_graph::build_dependency_graph_from_edges;
/// use sdi_graph::metrics::compute_metrics;
///
/// let dg = build_dependency_graph_from_edges(&[], &[]);
/// let m = compute_metrics(&dg);
/// assert_eq!(m.node_count, 0);
/// assert_eq!(m.edge_count, 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphMetrics {
    /// Total number of nodes (source files).
    pub node_count: usize,
    /// Total number of directed edges.
    pub edge_count: usize,
    /// Graph density: `edge_count / (node_count * (node_count - 1))`.
    /// Zero when `node_count < 2`.
    pub density: f64,
    /// Number of directed cycles detected via DFS; self-loops excluded.
    pub cycle_count: usize,
    /// Nodes with the highest out-degree, sorted descending.
    /// Contains at most `TOP_HUB_COUNT` entries.
    pub top_hubs: Vec<(PathBuf, usize)>,
    /// Number of weakly connected components.
    pub component_count: usize,
}

/// Maximum number of hubs to report.
const TOP_HUB_COUNT: usize = 10;

/// Computes [`GraphMetrics`] for a [`DependencyGraph`].
///
/// Cycle detection excludes self-loops. Connected components are computed as
/// weakly connected (direction ignored) via Kosaraju's algorithm on the
/// underlying directed graph.
pub fn compute_metrics(dg: &DependencyGraph) -> GraphMetrics {
    let n = dg.node_count();
    let e = dg.edge_count();

    let density = if n >= 2 {
        e as f64 / (n * (n - 1)) as f64
    } else {
        0.0
    };

    let top_hubs = compute_top_hubs(dg);
    let cycle_count = count_cycles(dg);
    let component_count = compute_component_count(dg);

    GraphMetrics {
        node_count: n,
        edge_count: e,
        density,
        cycle_count,
        top_hubs,
        component_count,
    }
}

/// Returns the top-`TOP_HUB_COUNT` nodes by out-degree, sorted descending.
fn compute_top_hubs(dg: &DependencyGraph) -> Vec<(PathBuf, usize)> {
    let mut degrees: Vec<(PathBuf, usize)> = (0..dg.node_count())
        .filter_map(|idx| {
            let ni = NodeIndex::new(idx);
            let path = dg.graph.node_weight(ni)?.clone();
            let out_deg = dg.graph.edges(ni).count();
            Some((path, out_deg))
        })
        .collect();

    degrees.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    degrees.truncate(TOP_HUB_COUNT);
    degrees
}

/// Counts directed cycles, excluding self-loops.
///
/// Uses a DFS-based back-edge detector. Each back edge found during DFS
/// contributes one cycle count.
fn count_cycles(dg: &DependencyGraph) -> usize {
    let n = dg.node_count();
    if n == 0 {
        return 0;
    }

    let mut visited = vec![false; n];
    let mut on_stack = vec![false; n];
    let mut cycles = 0;

    for start in 0..n {
        if !visited[start] {
            dfs_cycle_count(
                dg,
                start,
                &mut visited,
                &mut on_stack,
                &mut cycles,
            );
        }
    }
    cycles
}

/// DFS traversal that counts back edges (each = one cycle).
fn dfs_cycle_count(
    dg: &DependencyGraph,
    node: usize,
    visited: &mut Vec<bool>,
    on_stack: &mut Vec<bool>,
    cycles: &mut usize,
) {
    visited[node] = true;
    on_stack[node] = true;

    let ni = NodeIndex::new(node);
    // Collect neighbors into a sorted Vec for deterministic traversal.
    let mut neighbors: Vec<usize> = dg
        .graph
        .edges(ni)
        .filter_map(|e| {
            let tgt = e.target().index();
            // Skip self-loops.
            if tgt != node { Some(tgt) } else { None }
        })
        .collect();
    neighbors.sort_unstable();

    for neighbor in neighbors {
        if !visited[neighbor] {
            dfs_cycle_count(dg, neighbor, visited, on_stack, cycles);
        } else if on_stack[neighbor] {
            *cycles += 1;
        }
    }

    on_stack[node] = false;
}

/// Counts weakly connected components via union-find on the undirected edge set.
fn compute_component_count(dg: &DependencyGraph) -> usize {
    let n = dg.node_count();
    if n == 0 {
        return 0;
    }
    compute_wcc_count(dg, n)
}

/// Counts weakly connected components via union-find.
fn compute_wcc_count(dg: &DependencyGraph, n: usize) -> usize {
    let mut parent: Vec<usize> = (0..n).collect();

    fn find(parent: &mut Vec<usize>, x: usize) -> usize {
        if parent[x] != x {
            parent[x] = find(parent, parent[x]);
        }
        parent[x]
    }

    fn union(parent: &mut Vec<usize>, x: usize, y: usize) {
        let rx = find(parent, x);
        let ry = find(parent, y);
        if rx != ry {
            parent[rx] = ry;
        }
    }

    for (from, to) in dg.edges_as_pairs() {
        union(&mut parent, from, to);
    }

    let mut roots = std::collections::BTreeSet::new();
    for i in 0..n {
        roots.insert(find(&mut parent, i));
    }
    roots.len()
}
