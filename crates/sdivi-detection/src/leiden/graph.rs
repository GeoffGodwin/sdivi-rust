//! Internal graph representation for the Leiden hot path.

use sdivi_graph::DependencyGraph;
use std::collections::BTreeMap;

/// Compact undirected adjacency-list graph for the Leiden inner loop.
///
/// `edge_weights[i][j]` is the weight of the edge `adj[i][j]`.
/// For unweighted graphs (the default), all weights are `1.0`.
#[derive(Debug, Clone)]
pub(crate) struct LeidenGraph {
    /// Number of nodes.
    pub n: usize,
    /// `adj[i]` = neighbours of node `i` (sorted, deduplicated).
    pub adj: Vec<Vec<usize>>,
    /// `edge_weights[i][j]` = weight of edge from `i` to `adj[i][j]`.
    pub edge_weights: Vec<Vec<f64>>,
    /// `degree[i]` = sum of weights for all edges incident to `i`.
    pub degree: Vec<f64>,
    /// Half the sum of all weighted degrees (= total edge weight).
    pub total_weight: f64,
}

impl LeidenGraph {
    /// Builds an unweighted undirected graph from a `DependencyGraph`.
    pub fn from_dependency_graph(dg: &DependencyGraph) -> Self {
        let n = dg.node_count();
        Self::from_edges(n, &dg.edges_as_pairs())
    }

    /// Builds an undirected graph with optional per-edge weights from a `DependencyGraph`.
    ///
    /// `weight_map` keys are `(min_idx, max_idx)`; missing entries default to `1.0`.
    pub fn from_dependency_graph_weighted(
        dg: &DependencyGraph,
        weight_map: &BTreeMap<(usize, usize), f64>,
    ) -> Self {
        let n = dg.node_count();
        let edges: Vec<(usize, usize, f64)> = dg
            .edges_as_pairs()
            .into_iter()
            .map(|(u, v)| {
                let key = if u < v { (u, v) } else { (v, u) };
                let w = weight_map.get(&key).copied().unwrap_or(1.0);
                (u, v, w)
            })
            .collect();
        Self::from_edges_weighted(n, &edges)
    }

    /// Builds an unweighted undirected graph. All edge weights are `1.0`.
    pub fn from_edges(n: usize, edges: &[(usize, usize)]) -> Self {
        let edges_w: Vec<(usize, usize, f64)> = edges.iter().map(|&(u, v)| (u, v, 1.0)).collect();
        Self::from_edges_weighted(n, &edges_w)
    }

    /// Builds a weighted undirected graph from `(from, to, weight)` triples.
    ///
    /// Directed edges are symmetrised; self-loops are dropped. Duplicate
    /// undirected edges have their weights summed.
    pub fn from_edges_weighted(n: usize, edges: &[(usize, usize, f64)]) -> Self {
        // Accumulate weights per canonical (min, max) edge.
        let mut weight_acc: BTreeMap<(usize, usize), f64> = BTreeMap::new();
        for &(u, v, w) in edges {
            if u == v || u >= n || v >= n {
                continue;
            }
            let key = if u < v { (u, v) } else { (v, u) };
            *weight_acc.entry(key).or_insert(0.0) += w;
        }

        let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
        let mut edge_weights: Vec<Vec<f64>> = vec![vec![]; n];

        for (&(u, v), &w) in &weight_acc {
            adj[u].push(v);
            adj[v].push(u);
            edge_weights[u].push(w);
            edge_weights[v].push(w);
        }

        // Sort adj (and keep edge_weights parallel) for deterministic iteration.
        for i in 0..n {
            let mut pairs: Vec<(usize, f64)> = adj[i]
                .iter()
                .copied()
                .zip(edge_weights[i].iter().copied())
                .collect();
            pairs.sort_unstable_by_key(|&(nbr, _)| nbr);
            adj[i] = pairs.iter().map(|&(nbr, _)| nbr).collect();
            edge_weights[i] = pairs.iter().map(|&(_, w)| w).collect();
        }

        let degree: Vec<f64> = edge_weights.iter().map(|ws| ws.iter().sum()).collect();
        let total_weight: f64 = degree.iter().sum::<f64>() / 2.0;

        LeidenGraph {
            n,
            adj,
            edge_weights,
            degree,
            total_weight,
        }
    }

    /// Returns the weight of edge `(u, v)`, or `0.0` if the edge does not exist.
    ///
    /// `adj[u]` is sorted so this uses binary search — O(log degree(u)).
    #[allow(dead_code)] // utility method for future use / testing
    pub fn edge_weight(&self, u: usize, v: usize) -> f64 {
        match self.adj[u].binary_search(&v) {
            Ok(idx) => self.edge_weights[u][idx],
            Err(_) => 0.0,
        }
    }
}
