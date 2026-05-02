//! Refinement phase of the Leiden algorithm.
//!
//! For each community `C` in the coarse partition, applies a local move phase
//! restricted to the nodes of `C`, starting from singleton sub-communities.
//! The result is a finer partition (the "refined partition") that can discover
//! well-connected sub-communities that the coarse Louvain pass may have merged.

use rand::rngs::StdRng;
use rand::seq::SliceRandom as _;

use super::graph::LeidenGraph;
use super::modularity::edges_to_community;
use crate::partition::QualityFunction;

/// Produces a refined partition from a coarse `partition`.
///
/// Each community in `partition` is treated independently.  Within each
/// community, nodes start in singleton sub-communities and are greedily merged
/// if the move improves the quality function.
///
/// Returns an assignment vector of the same length as `partition` where each
/// entry is a refined community ID (globally numbered from 0).
pub(crate) fn refine_partition(
    graph: &LeidenGraph,
    partition: &[usize],
    rng: &mut StdRng,
    quality: &QualityFunction,
    gamma: f64,
) -> Vec<usize> {
    if graph.n == 0 {
        return vec![];
    }

    // Group nodes by their coarse community.
    let max_comm = partition.iter().copied().max().unwrap_or(0) + 1;
    let mut community_members: Vec<Vec<usize>> = vec![vec![]; max_comm];
    for (node, &comm) in partition.iter().enumerate() {
        community_members[comm].push(node);
    }

    // Start: each node in its own singleton (refined assignment = node index).
    let mut refined: Vec<usize> = (0..graph.n).collect();
    let mut next_comm_id = graph.n; // IDs for merged sub-communities.

    for members in &community_members {
        if members.len() <= 1 {
            continue; // Singleton community — nothing to refine.
        }

        refine_community(
            graph,
            members,
            &mut refined,
            &mut next_comm_id,
            rng,
            quality,
            gamma,
        );
    }

    // Re-number refined communities from 0..k.
    renumber_in_place(&mut refined);
    refined
}

/// Applies a local move phase within a single coarse community `members`.
fn refine_community(
    graph: &LeidenGraph,
    members: &[usize],
    refined: &mut [usize],
    next_id: &mut usize,
    rng: &mut StdRng,
    quality: &QualityFunction,
    gamma: f64,
) {
    // Shuffle for randomised, seed-deterministic processing.
    let mut order: Vec<usize> = members.to_vec();
    order.shuffle(rng);

    let member_set: std::collections::BTreeSet<usize> = members.iter().copied().collect();

    let max_iter = 10; // Limit inner iterations for the refinement pass.
    for _ in 0..max_iter {
        let mut moved = false;
        for &node in &order {
            moved |= try_merge_node(graph, node, &member_set, refined, next_id, quality, gamma);
        }
        if !moved {
            break;
        }
    }
}

/// Tries to merge `node` into a neighbouring sub-community within `member_set`.
///
/// Returns `true` if the node was moved.
fn try_merge_node(
    graph: &LeidenGraph,
    node: usize,
    member_set: &std::collections::BTreeSet<usize>,
    refined: &mut [usize],
    _next_id: &mut usize,
    quality: &QualityFunction,
    gamma: f64,
) -> bool {
    let current_comm = refined[node];

    // Candidate communities: sub-communities of neighbours within this coarse
    // community that are different from `node`'s current sub-community.
    let candidates: std::collections::BTreeSet<usize> = graph.adj[node]
        .iter()
        .filter(|&&nbr| member_set.contains(&nbr) && refined[nbr] != current_comm)
        .map(|&nbr| refined[nbr])
        .collect();

    if candidates.is_empty() {
        return false;
    }

    let best_comm = best_candidate(graph, node, &candidates, refined, quality, gamma);

    match best_comm {
        Some((comm, gain)) if gain > 1e-10 => {
            move_node_to(node, comm, refined);
            true
        }
        _ => false,
    }
}

/// Finds the candidate community with the highest positive move gain.
fn best_candidate(
    graph: &LeidenGraph,
    node: usize,
    candidates: &std::collections::BTreeSet<usize>,
    refined: &[usize],
    quality: &QualityFunction,
    gamma: f64,
) -> Option<(usize, f64)> {
    let m = graph.total_weight;
    if m == 0.0 {
        return None;
    }

    let mut best: Option<(usize, f64)> = None;

    for &comm in candidates {
        let k_in = edges_to_community(graph, node, comm, refined);
        let gain = match quality {
            QualityFunction::Modularity => {
                let sigma: f64 = graph.adj[node]
                    .iter()
                    .filter(|&&nbr| refined[nbr] == comm)
                    .count() as f64; // approximate sigma_tot for sub-comm
                k_in - graph.degree[node] * sigma / (2.0 * m)
            }
            QualityFunction::Cpm { gamma: _ } => {
                let n_comm = refined.iter().filter(|&&c| c == comm).count() as f64;
                k_in - gamma * n_comm
            }
        };

        if gain > best.map(|(_, g)| g).unwrap_or(f64::NEG_INFINITY) {
            best = Some((comm, gain));
        }
    }

    best
}

/// Moves `node` to `to` community.
fn move_node_to(node: usize, to: usize, refined: &mut [usize]) {
    refined[node] = to;
}

/// Renumbers community IDs in-place to a dense range `[0, k)`.
fn renumber_in_place(assignment: &mut [usize]) {
    let mut map = std::collections::BTreeMap::new();
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
