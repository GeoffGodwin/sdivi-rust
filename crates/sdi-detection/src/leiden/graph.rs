//! Internal graph representation for the Leiden hot path.
//!
//! [`LeidenGraph`] is built once from a [`DependencyGraph`] at the start of
//! a Leiden run and dropped when the run completes.  It uses a plain
//! adjacency-list representation (Vec of Vecs) which is faster to traverse in
//! the tight inner loop than petgraph's internal structures.
//!
//! Directed dependency edges are symmetrised: a directed A→B edge becomes
//! undirected A–B for community detection purposes.

use sdi_graph::DependencyGraph;

/// Compact undirected adjacency-list graph for the Leiden inner loop.
///
/// All edge weights are `1.0` (unweighted graph, `weighted_edges = false`
/// per `[boundaries]` config).
#[derive(Debug, Clone)]
pub(crate) struct LeidenGraph {
    /// Number of nodes.
    pub n: usize,
    /// `adj[i]` = neighbours of node `i` (unsorted, may contain duplicates
    /// after symmetrisation — deduplication is done during construction).
    pub adj: Vec<Vec<usize>>,
    /// `degree[i]` = number of unique neighbours of node `i`.
    pub degree: Vec<f64>,
    /// Half the sum of all degrees (= number of undirected edges).
    pub total_weight: f64,
}

impl LeidenGraph {
    /// Builds an undirected `LeidenGraph` from a `DependencyGraph` by
    /// symmetrising directed edges and deduplicating.
    pub fn from_dependency_graph(dg: &DependencyGraph) -> Self {
        let n = dg.node_count();
        Self::from_edges(n, &dg.edges_as_pairs())
    }

    /// Builds an undirected `LeidenGraph` from an explicit edge list.
    ///
    /// Edges are symmetrised; self-loops and duplicate undirected edges are
    /// removed.
    pub fn from_edges(n: usize, edges: &[(usize, usize)]) -> Self {
        let mut adj: Vec<Vec<usize>> = vec![vec![]; n];

        for &(u, v) in edges {
            if u == v || u >= n || v >= n {
                continue;
            }
            adj[u].push(v);
            adj[v].push(u);
        }

        // Deduplicate and sort for deterministic iteration order.
        for nbrs in adj.iter_mut() {
            nbrs.sort_unstable();
            nbrs.dedup();
        }

        let degree: Vec<f64> = adj.iter().map(|nbrs| nbrs.len() as f64).collect();
        let total_weight: f64 = degree.iter().sum::<f64>() / 2.0;

        LeidenGraph { n, adj, degree, total_weight }
    }
}
