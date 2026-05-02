//! Internal graph representation for the Leiden hot path.

use sdivi_graph::DependencyGraph;
use std::collections::BTreeMap;

/// Compact undirected adjacency-list graph for the Leiden inner loop.
///
/// `edge_weights[i][j]` is the weight of the edge `adj[i][j]`.
/// For unweighted graphs (the default), all weights are `1.0`.
///
/// # Self-loops
///
/// A self-loop on node `u` with weight `w` is stored in `self_loops[u]`,
/// not in `adj[u]`.  A self-loop contributes `2w` to `degree[u]` (standard
/// undirected-graph convention: both endpoints are `u`) and `w` to
/// `total_weight`.
///
/// This type is `#[doc(hidden)]` public for integration-test plumbing only.
/// It is not part of the stable API.
#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct LeidenGraph {
    /// Number of nodes.
    pub n: usize,
    /// `adj[i]` = neighbours of node `i` (sorted, deduplicated). Self-loops excluded.
    pub adj: Vec<Vec<usize>>,
    /// `edge_weights[i][j]` = weight of edge from `i` to `adj[i][j]`.
    pub edge_weights: Vec<Vec<f64>>,
    /// `degree[i]` = sum of cross-edge weights + `2 × self_loops[i]`.
    ///
    /// The `2×` factor follows the standard undirected-graph convention: a
    /// self-loop of weight `w` has both endpoints at `u`, contributing `w`
    /// per endpoint.
    pub degree: Vec<f64>,
    /// Total edge weight: sum of all cross-edge weights + sum of all self-loop weights.
    ///
    /// Note: this equals `(degree.sum() - self_loops.sum()) / 2 + self_loops.sum()`.
    /// Do **not** use `degree.sum() / 2` once self-loops exist — that would
    /// double-count self-loops.
    pub total_weight: f64,
    /// `self_loops[i]` = total weight of all self-loop edges on node `i`.
    ///
    /// Zero for nodes with no self-loops (the common case in non-aggregate graphs).
    /// Parallel self-loop edges `(u, u, w1)` and `(u, u, w2)` accumulate to
    /// `self_loops[u] = w1 + w2`.
    pub self_loops: Vec<f64>,
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
    ///
    /// Self-loops in `edges` are preserved with weight `1.0`.
    pub fn from_edges(n: usize, edges: &[(usize, usize)]) -> Self {
        let edges_w: Vec<(usize, usize, f64)> = edges.iter().map(|&(u, v)| (u, v, 1.0)).collect();
        Self::from_edges_weighted(n, &edges_w)
    }

    /// Builds a weighted undirected graph from `(from, to, weight)` triples.
    ///
    /// Directed edges are symmetrised; duplicate undirected edges have their
    /// weights summed.  Self-loops `(u, u, w)` are preserved in `self_loops[u]`
    /// rather than dropped — they contribute to `degree[u]` and `total_weight`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sdivi_detection::internal::LeidenGraph;
    /// // 1-node graph with a single self-loop of weight 2.0.
    /// let g = LeidenGraph::from_edges_weighted(1, &[(0, 0, 2.0)]);
    /// assert_eq!(g.self_loops[0], 2.0);
    /// // Self-loop contributes 2w to degree (both endpoints are u).
    /// assert_eq!(g.degree[0], 4.0);
    /// assert_eq!(g.total_weight, 2.0);
    /// assert!(g.adj[0].is_empty(), "self-loop is not in adj");
    /// ```
    pub fn from_edges_weighted(n: usize, edges: &[(usize, usize, f64)]) -> Self {
        // Accumulate cross-edge weights per canonical (min, max) pair.
        let mut weight_acc: BTreeMap<(usize, usize), f64> = BTreeMap::new();
        // Accumulate self-loop weights per node.
        let mut self_loop_acc: Vec<f64> = vec![0.0; n];

        for &(u, v, w) in edges {
            if u >= n || v >= n {
                continue;
            }
            if u == v {
                // Self-loop: accumulate into self_loops, not into adj.
                self_loop_acc[u] += w;
            } else {
                let key = if u < v { (u, v) } else { (v, u) };
                *weight_acc.entry(key).or_insert(0.0) += w;
            }
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

        // degree[i] = cross-edge weight sum + 2 × self_loop weight.
        // The 2× factor: both endpoints of a self-loop are `i`.
        let degree: Vec<f64> = edge_weights
            .iter()
            .zip(self_loop_acc.iter())
            .map(|(ws, &sl)| ws.iter().sum::<f64>() + 2.0 * sl)
            .collect();

        // total_weight = cross-edge total + self-loop total.
        // Do NOT use degree.sum() / 2 — that double-counts self-loops.
        let cross_edge_total: f64 = weight_acc.values().sum();
        let self_loop_total: f64 = self_loop_acc.iter().sum();
        let total_weight = cross_edge_total + self_loop_total;

        LeidenGraph {
            n,
            adj,
            edge_weights,
            degree,
            total_weight,
            self_loops: self_loop_acc,
        }
    }

    /// Returns the weight of edge `(u, v)`, or `0.0` if the edge does not exist.
    ///
    /// When `u == v`, returns the self-loop weight `self_loops[u]`.
    /// For cross-edges, `adj[u]` is sorted so this uses binary search — O(log degree(u)).
    #[allow(dead_code)] // utility method for testing
    pub fn edge_weight(&self, u: usize, v: usize) -> f64 {
        if u == v {
            return self.self_loops[u];
        }
        match self.adj[u].binary_search(&v) {
            Ok(idx) => self.edge_weights[u][idx],
            Err(_) => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_node_self_loop() {
        let g = LeidenGraph::from_edges_weighted(1, &[(0, 0, 2.0)]);
        assert_eq!(g.self_loops[0], 2.0);
        assert_eq!(
            g.degree[0], 4.0,
            "self-loop weight 2.0 contributes 2×2.0=4.0 to degree"
        );
        assert_eq!(g.total_weight, 2.0);
        assert!(g.adj[0].is_empty(), "self-loop must not appear in adj");
    }

    #[test]
    fn two_node_cross_edge_plus_self_loops() {
        // cross-edge (0,1) weight 1.0; self-loops on both nodes weight 0.5 each
        let g = LeidenGraph::from_edges_weighted(2, &[(0, 0, 0.5), (1, 1, 0.5), (0, 1, 1.0)]);
        assert_eq!(g.degree[0], 2.0, "1.0 cross + 2×0.5 self = 2.0");
        assert_eq!(g.degree[1], 2.0);
        assert_eq!(g.total_weight, 2.0, "1.0 cross + 0.5 + 0.5 self = 2.0");
        assert_eq!(g.self_loops[0], 0.5);
        assert_eq!(g.self_loops[1], 0.5);
    }

    #[test]
    fn self_loop_accumulation() {
        // Parallel self-loops on node 0 should accumulate.
        let g = LeidenGraph::from_edges_weighted(1, &[(0, 0, 1.0), (0, 0, 2.0)]);
        assert_eq!(g.self_loops[0], 3.0);
        assert_eq!(g.degree[0], 6.0);
        assert_eq!(g.total_weight, 3.0);
    }

    #[test]
    fn edge_weight_self_loop() {
        let g = LeidenGraph::from_edges_weighted(2, &[(0, 0, 1.5), (0, 1, 2.0)]);
        assert_eq!(
            g.edge_weight(0, 0),
            1.5,
            "edge_weight(u,u) returns self_loops[u]"
        );
        assert_eq!(g.edge_weight(0, 1), 2.0);
        assert_eq!(g.edge_weight(1, 0), 2.0);
        assert_eq!(g.edge_weight(1, 1), 0.0, "no self-loop on node 1");
    }

    #[test]
    fn no_self_loops_unchanged() {
        // Verify that graphs without self-loops behave identically to before.
        let g = LeidenGraph::from_edges_weighted(3, &[(0, 1, 1.0), (1, 2, 1.0)]);
        assert_eq!(g.self_loops, vec![0.0, 0.0, 0.0]);
        assert_eq!(g.degree, vec![1.0, 2.0, 1.0]);
        assert_eq!(g.total_weight, 2.0);
    }
}
