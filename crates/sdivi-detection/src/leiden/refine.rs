//! Refinement phase of the Leiden algorithm.
//!
//! For each community `C` in the coarse partition, applies a local move phase
//! restricted to the nodes of `C`, starting from singleton sub-communities.
//! The result is a finer partition (the "refined partition") that can discover
//! well-connected sub-communities that the coarse Louvain pass may have merged.
//!
//! Uses real per-sub-community `Σ_tot` (sum of degrees) rather than a count
//! fudge, matching the ΔQ formula from Traag et al. 2019.

use std::collections::BTreeMap;

use rand::rngs::StdRng;
use rand::seq::SliceRandom as _;

use super::graph::LeidenGraph;
use crate::partition::QualityFunction;

const MAX_REFINE_ITER: usize = 10;

/// Per-sub-community statistics for the refinement phase.
///
/// Mirrors `ModularityState` but uses singleton initialisation (each node
/// starts as its own sub-community) rather than a prior-partition init.
/// Kept separate from `ModularityState` to avoid confusion between the
/// coarse-level state and the refinement-level state.
#[derive(Debug, Clone)]
pub struct RefinementState {
    /// `assignment[i]` — sub-community ID of node `i`.  Initially `i`.
    pub assignment: Vec<usize>,
    /// `sigma_tot[c]` — sum of degrees of nodes currently in sub-community `c`.
    pub sigma_tot: Vec<f64>,
    /// `inner_edges[c]` — sum of intra-sub-community edge weights for `c`.
    pub inner_edges: Vec<f64>,
    /// `size[c]` — number of nodes currently in sub-community `c`.
    pub size: Vec<usize>,
}

impl RefinementState {
    /// Constructs refinement state from an arbitrary `partition`.
    ///
    /// `capacity` controls the size of the internal arrays; the resulting
    /// state can hold communities with IDs in `[0, capacity)`.  When called
    /// with the singleton partition `(0..n).collect()`, every node starts
    /// in its own sub-community.
    pub fn from_partition(graph: &LeidenGraph, partition: &[usize], capacity: usize) -> Self {
        let n = graph.n;
        debug_assert_eq!(partition.len(), n);
        let max_id = partition.iter().copied().max().map(|m| m + 1).unwrap_or(0);
        let cap = capacity.max(max_id);

        let mut sigma_tot = vec![0.0f64; cap];
        let mut inner_edges = vec![0.0f64; cap];
        let mut size = vec![0usize; cap];

        for (i, &c) in partition.iter().enumerate().take(n) {
            sigma_tot[c] += graph.degree[i];
            size[c] += 1;
        }
        for i in 0..n {
            let c_i = partition[i];
            inner_edges[c_i] += graph.self_loops[i];
            for (idx, &j) in graph.adj[i].iter().enumerate() {
                if partition[j] == c_i && j > i {
                    inner_edges[c_i] += graph.edge_weights[i][idx];
                }
            }
        }

        RefinementState {
            assignment: partition.to_vec(),
            sigma_tot,
            inner_edges,
            size,
        }
    }

    /// Moves `node` from sub-community `from` to sub-community `to`.
    ///
    /// Updates `assignment`, `sigma_tot`, `inner_edges`, and `size`.
    /// Must be called while `assignment[node]` still equals `from`.
    pub fn apply_move(&mut self, graph: &LeidenGraph, node: usize, from: usize, to: usize) {
        for (idx, &nbr) in graph.adj[node].iter().enumerate() {
            let w = graph.edge_weights[node][idx];
            let nbr_comm = self.assignment[nbr];
            if nbr_comm == from {
                self.inner_edges[from] -= w;
            } else if nbr_comm == to {
                self.inner_edges[to] += w;
            }
        }
        // Self-loop moves with the node.
        self.inner_edges[from] -= graph.self_loops[node];
        self.inner_edges[to] += graph.self_loops[node];

        self.sigma_tot[from] -= graph.degree[node];
        self.sigma_tot[to] += graph.degree[node];
        self.size[from] -= 1;
        self.size[to] += 1;
        self.assignment[node] = to;
    }

    /// Modularity gain for moving `node` into sub-community `to`.
    ///
    /// `k_in_to` is the sum of weights of edges from `node` to nodes
    /// currently in `to`.  Self-loops cancel in the ΔQ derivation.
    pub fn move_gain(&self, graph: &LeidenGraph, node: usize, to: usize, k_in_to: f64) -> f64 {
        let sigma_to = self.sigma_tot[to];
        let m2 = 2.0 * graph.total_weight;
        if m2 == 0.0 {
            return 0.0;
        }
        k_in_to - graph.degree[node] * sigma_to / m2
    }
}

/// Checks the γ-connectivity threshold before absorbing a node into a
/// sub-community (v0 simplification of Traag 2019 §2.2, Algorithm 2).
///
/// The exact Traag formulation checks `E(C) ≥ γ · |C| · (|S| − |C|) / (2 m_S)`.
/// This v0 proxy uses the bound `k_in_to ≥ γ · (|C| − |C|² / |S|)`.
///
/// Always returns `true` when `gamma == 0.0`.  With `gamma = 1.0` (the
/// default), this rejects moves where the node is weakly connected to
/// `candidate` relative to the sub-community's current size.
///
/// # Note
///
/// This is the v0 simplification. See Traag 2019 Algorithm 2 for the exact
/// formulation.  The simplified check passes the verify-leiden fixture suite
/// (1 % modularity tolerance).
pub fn well_connected(k_in_to: f64, size_candidate: usize, size_s: usize, gamma: f64) -> bool {
    if gamma == 0.0 || size_s == 0 {
        return true;
    }
    let sc = size_candidate as f64;
    let ss = size_s as f64;
    let threshold = gamma * (sc - sc * sc / ss);
    k_in_to >= threshold
}

/// Produces a refined partition from a coarse `partition`.
///
/// Each community in `partition` is treated independently via an induced
/// subgraph, giving O(|members|) setup cost instead of O(n).  Within each
/// community, nodes start in singleton sub-communities and are greedily merged
/// if the move improves the quality function.
///
/// Returns an assignment vector of the same length as `partition` where each
/// entry is a refined community ID (globally numbered from 0).
///
/// This function is `#[doc(hidden)]` public for integration-test plumbing only.
#[doc(hidden)]
pub fn refine_partition(
    graph: &LeidenGraph,
    partition: &[usize],
    rng: &mut StdRng,
    quality: &QualityFunction,
    gamma: f64,
) -> Vec<usize> {
    if graph.n == 0 {
        return vec![];
    }

    let max_comm = partition.iter().copied().max().unwrap_or(0) + 1;
    let mut community_members: Vec<Vec<usize>> = vec![vec![]; max_comm];
    for (node, &comm) in partition.iter().enumerate() {
        community_members[comm].push(node);
    }

    // Start: each node in its own singleton (refined assignment = node index).
    let mut refined: Vec<usize> = (0..graph.n).collect();

    for members in &community_members {
        if members.len() <= 1 {
            continue;
        }
        refine_community(graph, members, &mut refined, rng, quality, gamma);
    }

    renumber_in_place(&mut refined);
    refined
}

/// Applies a local move phase within a single coarse community `members`.
fn refine_community(
    graph: &LeidenGraph,
    members: &[usize],
    refined: &mut [usize],
    rng: &mut StdRng,
    quality: &QualityFunction,
    gamma: f64,
) {
    let (sub, local_to_global) = graph.induced_subgraph(members);
    let local_n = sub.n;
    let size_s = local_n;

    // State sized to sub.n, not graph.n — O(|members|) setup instead of O(n).
    let singleton: Vec<usize> = (0..local_n).collect();
    let mut state = RefinementState::from_partition(&sub, &singleton, local_n);

    let mut order: Vec<usize> = (0..local_n).collect();
    order.shuffle(rng);

    for _ in 0..MAX_REFINE_ITER {
        let mut moved = false;
        for &node in &order {
            let current_comm = state.assignment[node];

            // No in_coarse filter needed — sub contains only members.
            let mut k_in_per_comm: BTreeMap<usize, f64> = BTreeMap::new();
            for (idx, &nbr) in sub.adj[node].iter().enumerate() {
                let nbr_comm = state.assignment[nbr];
                if nbr_comm == current_comm {
                    continue;
                }
                *k_in_per_comm.entry(nbr_comm).or_insert(0.0) += sub.edge_weights[node][idx];
            }

            if k_in_per_comm.is_empty() {
                continue;
            }

            let mut best_comm = current_comm;
            let mut best_gain = 0.0f64;

            for (&comm, &k_in_to) in &k_in_per_comm {
                let size_candidate = state.size[comm];
                if !well_connected(k_in_to, size_candidate, size_s, gamma) {
                    continue;
                }
                let gain = match quality {
                    QualityFunction::Modularity => state.move_gain(&sub, node, comm, k_in_to),
                    QualityFunction::Cpm { gamma: _ } => k_in_to - gamma * state.size[comm] as f64,
                };
                if gain > best_gain {
                    best_gain = gain;
                    best_comm = comm;
                }
            }

            if best_gain > 1e-10 {
                state.apply_move(&sub, node, current_comm, best_comm);
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }

    // Map each local sub-community to a globally unique ID using the smallest
    // global member of that sub-community.  Prevents ID collisions between sibling
    // refine_community calls, which operate on disjoint member sets.
    let mut sc_to_global: BTreeMap<usize, usize> = BTreeMap::new();
    for (local, &sc) in state.assignment.iter().enumerate() {
        let global = local_to_global[local];
        let entry = sc_to_global.entry(sc).or_insert(global);
        if global < *entry {
            *entry = global;
        }
    }

    for (local, &sc) in state.assignment.iter().enumerate() {
        refined[local_to_global[local]] = sc_to_global[&sc];
    }
}

/// Renumbers community IDs in-place to a dense range `[0, k)`.
fn renumber_in_place(assignment: &mut [usize]) {
    let mut map = BTreeMap::new();
    let mut next = 0usize;
    for comm in assignment.iter_mut() {
        let entry = map.entry(*comm).or_insert_with(|| {
            let c = next;
            next += 1;
            c
        });
        *comm = *entry;
    }
}
