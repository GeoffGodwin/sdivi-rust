//! Modularity quality function for the Leiden algorithm.
//!
//! Implements the Newman–Girvan modularity:
//! `Q = Σ_C [L_C/m − (Σ_C / 2m)²]`
//!
//! where `L_C` is the number of internal edges in community `C`, `Σ_C` is the
//! sum of degrees of nodes in `C`, and `m` is the total number of (undirected)
//! edges.

use super::graph::LeidenGraph;

/// Per-community statistics needed for O(1) modularity gain computations.
///
/// After every node move, only two communities change — making incremental
/// updates O(degree(node)) rather than O(n).
#[derive(Debug, Clone)]
pub(crate) struct ModularityState {
    /// `assignment[node]` = current community of `node`.
    pub assignment: Vec<usize>,
    /// `sigma_tot[c]` = sum of degrees of nodes in community `c`.
    pub sigma_tot: Vec<f64>,
    /// `inner_edges[c]` = number of edges with both endpoints in community `c`.
    pub inner_edges: Vec<f64>,
    /// `size[c]` = number of nodes in community `c`.
    pub size: Vec<usize>,
}

impl ModularityState {
    /// Initialises the state from an external assignment vector.
    ///
    /// `max_comm` must be `>= max(assignment) + 1`.
    ///
    /// Arrays are allocated with size `max(max_comm, n)` so that node indices
    /// can safely be used as singleton community IDs after removal.
    pub fn from_assignment(
        graph: &LeidenGraph,
        assignment: Vec<usize>,
        max_comm: usize,
    ) -> Self {
        let n = graph.n;
        // Capacity must accommodate node indices used as singleton IDs.
        let capacity = max_comm.max(n);
        let mut sigma_tot = vec![0.0f64; capacity];
        let mut inner_edges = vec![0.0f64; capacity];
        let mut size = vec![0usize; capacity];

        for i in 0..n {
            let c = assignment[i];
            sigma_tot[c] += graph.degree[i];
            size[c] += 1;
        }

        for i in 0..n {
            let c_i = assignment[i];
            for &j in &graph.adj[i] {
                if assignment[j] == c_i && j > i {
                    inner_edges[c_i] += 1.0;
                }
            }
        }

        ModularityState { assignment, sigma_tot, inner_edges, size }
    }

    /// Computes the gain of moving `node` to community `to`.
    ///
    /// Returns the *unnormalised* gain (divide by `m` to get `ΔQ`).
    /// Positive value means the move improves modularity.
    ///
    /// The caller must have already conceptually removed `node` from its
    /// current community (i.e., `sigma_tot[current]` must NOT include
    /// `node`'s degree).
    pub fn move_gain(
        &self,
        graph: &LeidenGraph,
        node: usize,
        to: usize,
        k_in_to: f64,
    ) -> f64 {
        let k_v = graph.degree[node];
        let sigma_to = self.sigma_tot[to];
        let m2 = 2.0 * graph.total_weight;
        if m2 == 0.0 {
            return 0.0;
        }
        k_in_to - k_v * sigma_to / m2
    }

    /// Removes `node` from its current community in place.
    ///
    /// Returns the number of edges from `node` to its (old) community.
    pub fn remove_node(&mut self, graph: &LeidenGraph, node: usize) -> f64 {
        let comm = self.assignment[node];
        let mut k_in = 0.0f64;
        for &nbr in &graph.adj[node] {
            if self.assignment[nbr] == comm {
                k_in += 1.0;
                self.inner_edges[comm] -= 1.0;
            }
        }
        self.sigma_tot[comm] -= graph.degree[node];
        self.size[comm] -= 1;
        // Place node in its own singleton (uses its own index as community).
        self.assignment[node] = node;
        self.sigma_tot[node] = graph.degree[node];
        self.size[node] = 1;
        k_in
    }

    /// Adds `node` to community `to`.
    pub fn add_node(&mut self, graph: &LeidenGraph, node: usize, to: usize) {
        let mut k_in = 0.0f64;
        for &nbr in &graph.adj[node] {
            if self.assignment[nbr] == to {
                k_in += 1.0;
                self.inner_edges[to] += 1.0;
            }
        }
        self.sigma_tot[to] += graph.degree[node];
        self.size[to] += 1;
        self.assignment[node] = to;
        let _ = k_in; // used via inner_edges update above
    }

}

/// Counts edges from `node` to community `to` using the current assignment.
pub(crate) fn edges_to_community(
    graph: &LeidenGraph,
    node: usize,
    to: usize,
    assignment: &[usize],
) -> f64 {
    graph.adj[node]
        .iter()
        .filter(|&&nbr| assignment[nbr] == to)
        .count() as f64
}
