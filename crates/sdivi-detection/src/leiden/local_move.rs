//! Louvain-style local move phase for the Leiden algorithm.

use std::collections::BTreeMap;

use rand::rngs::StdRng;
use rand::seq::SliceRandom as _;

use super::cpm;
use super::graph::LeidenGraph;
use super::modularity::ModularityState;
use crate::partition::QualityFunction;

/// One pass of the Louvain-style local move phase.
///
/// Visits nodes in a random (seed-deterministic) order and moves each node to
/// the neighbour community that maximises the quality function gain.
///
/// Returns `true` if at least one node was moved.
pub(super) fn local_move_phase(
    graph: &LeidenGraph,
    partition: &mut Vec<usize>,
    rng: &mut StdRng,
    quality: &QualityFunction,
    gamma: f64,
) -> bool {
    let n = graph.n;

    // Offset IDs by n to prevent collisions with ModularityState singleton IDs,
    // avoiding size-counter underflow when a node index matches a community ID.
    let offset_partition: Vec<usize> = partition.iter().map(|&c| c + n).collect();
    let max_comm = offset_partition.iter().copied().max().unwrap_or(0) + 1;

    let mut state = ModularityState::from_assignment(graph, offset_partition, max_comm);

    let mut order: Vec<usize> = (0..n).collect();
    order.shuffle(rng);

    let mut any_moved = false;

    for &node in &order {
        let old_comm = state.assignment[node];

        let _k_in_old = state.remove_node(graph, node);
        let (best_comm, best_gain) = best_neighbour_community(graph, node, &state, quality, gamma);

        if best_gain > 1e-10 {
            state.add_node(graph, node, best_comm);
            any_moved = true;
        } else {
            // node index is always < n; offset community IDs are >= n, so this singleton ID cannot collide
            let target = if state.size[old_comm] == 0 {
                node
            } else {
                old_comm
            };
            state.add_node(graph, node, target);
        }
    }

    *partition = state.assignment.clone();
    any_moved
}

/// Finds the neighbour community with the highest positive move gain.
///
/// Returns `(best_community_id, best_gain)`.
fn best_neighbour_community(
    graph: &LeidenGraph,
    node: usize,
    state: &ModularityState,
    quality: &QualityFunction,
    gamma: f64,
) -> (usize, f64) {
    // Accumulate edge weights per candidate community.
    let mut k_in_per_comm: BTreeMap<usize, f64> = BTreeMap::new();
    for (idx, &nbr) in graph.adj[node].iter().enumerate() {
        let c = state.assignment[nbr];
        *k_in_per_comm.entry(c).or_insert(0.0) += graph.edge_weights[node][idx];
    }

    let mut best_comm = node; // default: singleton
    let mut best_gain = 0.0f64;

    for (comm, k_in) in k_in_per_comm {
        let gain = compute_gain(graph, node, comm, k_in, state, quality, gamma);
        if gain > best_gain {
            best_gain = gain;
            best_comm = comm;
        }
    }

    (best_comm, best_gain)
}

/// Computes the move gain for a given quality function.
fn compute_gain(
    graph: &LeidenGraph,
    node: usize,
    comm: usize,
    k_in: f64,
    state: &ModularityState,
    quality: &QualityFunction,
    gamma: f64,
) -> f64 {
    match quality {
        QualityFunction::Modularity => state.move_gain(graph, node, comm, k_in),
        QualityFunction::Cpm { gamma: _ } => {
            let n_comm = state.size[comm] as f64;
            cpm::cpm_move_gain(k_in, n_comm, gamma)
        }
    }
}
