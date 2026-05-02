//! Modularity quality function for the Leiden algorithm.

use super::graph::LeidenGraph;

/// Per-community statistics needed for O(1) modularity gain computations.
#[derive(Debug, Clone)]
pub(crate) struct ModularityState {
    pub assignment: Vec<usize>,
    pub sigma_tot: Vec<f64>,
    pub inner_edges: Vec<f64>,
    pub size: Vec<usize>,
}

impl ModularityState {
    pub fn from_assignment(graph: &LeidenGraph, assignment: Vec<usize>, max_comm: usize) -> Self {
        let n = graph.n;
        let capacity = max_comm.max(n);
        let mut sigma_tot = vec![0.0f64; capacity];
        let mut inner_edges = vec![0.0f64; capacity];
        let mut size = vec![0usize; capacity];

        for (i, &c) in assignment.iter().enumerate().take(n) {
            sigma_tot[c] += graph.degree[i];
            size[c] += 1;
        }

        for i in 0..n {
            let c_i = assignment[i];
            // Self-loops are always internal to node i's own community.
            inner_edges[c_i] += graph.self_loops[i];
            for (idx, &j) in graph.adj[i].iter().enumerate() {
                if assignment[j] == c_i && j > i {
                    inner_edges[c_i] += graph.edge_weights[i][idx];
                }
            }
        }

        ModularityState {
            assignment,
            sigma_tot,
            inner_edges,
            size,
        }
    }

    /// Computes the modularity gain for moving `node` from its singleton to community `to`.
    ///
    /// The self-loop term cancels exactly in the ΔQ derivation (the loop is
    /// internal to `v`'s community both before and after the move), so this
    /// method is unchanged from the no-self-loop version.
    pub fn move_gain(&self, graph: &LeidenGraph, node: usize, to: usize, k_in_to: f64) -> f64 {
        let k_v = graph.degree[node];
        let sigma_to = self.sigma_tot[to];
        let m2 = 2.0 * graph.total_weight;
        if m2 == 0.0 {
            return 0.0;
        }
        k_in_to - k_v * sigma_to / m2
    }

    pub fn remove_node(&mut self, graph: &LeidenGraph, node: usize) -> f64 {
        let comm = self.assignment[node];
        let mut k_in = 0.0f64;
        for (idx, &nbr) in graph.adj[node].iter().enumerate() {
            if self.assignment[nbr] == comm {
                let w = graph.edge_weights[node][idx];
                k_in += w;
                self.inner_edges[comm] -= w;
            }
        }
        // Self-loop leaves the old community when the node becomes a singleton.
        self.inner_edges[comm] -= graph.self_loops[node];
        self.sigma_tot[comm] -= graph.degree[node];
        self.size[comm] -= 1;
        self.assignment[node] = node;
        self.sigma_tot[node] = graph.degree[node];
        self.size[node] = 1;
        // Singleton's only internal edges are its self-loops.
        self.inner_edges[node] = graph.self_loops[node];
        k_in
    }

    pub fn add_node(&mut self, graph: &LeidenGraph, node: usize, to: usize) {
        // Clear the singleton slot that node is leaving.  When `to == node`
        // this is immediately overwritten by the self-loop addition below,
        // producing the correct singleton value.
        self.inner_edges[node] = 0.0;
        for (idx, &nbr) in graph.adj[node].iter().enumerate() {
            if self.assignment[nbr] == to {
                let w = graph.edge_weights[node][idx];
                self.inner_edges[to] += w;
            }
        }
        // Self-loop joins the new community.
        self.inner_edges[to] += graph.self_loops[node];
        self.sigma_tot[to] += graph.degree[node];
        self.size[to] += 1;
        self.assignment[node] = to;
    }
}
