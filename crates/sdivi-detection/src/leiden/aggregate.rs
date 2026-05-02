//! Network aggregation phase of the Leiden algorithm.

use std::collections::BTreeMap;

use super::graph::LeidenGraph;

#[doc(hidden)]
pub struct AggregateResult {
    pub graph: LeidenGraph,
    pub membership: Vec<Vec<usize>>,
}

/// Aggregates `graph` into a coarser graph where each community in
/// `refined_partition` becomes a super-node.
///
/// **Correctness invariants:**
/// - Intra-community cross-edges become self-loops on the super-node, preserving
///   the strong intra-community pull that modularity depends on.
/// - Each undirected cross-edge is visited once (upper-triangle walk), so
///   inter-community edge weights are not double-counted.
/// - Source node self-loops propagate to the aggregate super-node regardless of
///   partition.
///
/// This function is `#[doc(hidden)]` public for integration-test plumbing only.
#[doc(hidden)]
pub fn aggregate_network(graph: &LeidenGraph, refined_partition: &[usize]) -> AggregateResult {
    let n = graph.n;
    let num_comms = refined_partition
        .iter()
        .copied()
        .max()
        .map(|m| m + 1)
        .unwrap_or(0);

    let mut membership: Vec<Vec<usize>> = vec![vec![]; num_comms];
    for (node, &comm) in refined_partition.iter().enumerate() {
        membership[comm].push(node);
    }

    // edge_map accumulates both cross-edges (u,v with u<v) and self-loops (u,u).
    let mut edge_map: BTreeMap<(usize, usize), f64> = BTreeMap::new();

    for u in 0..n {
        let cu = refined_partition[u];

        // Walk only the upper triangle (v > u) — each undirected cross-edge
        // is visited exactly once, eliminating the prior 2× double-count.
        for (idx, &v) in graph.adj[u].iter().enumerate() {
            if v < u {
                continue;
            }
            let cv = refined_partition[v];
            let w = graph.edge_weights[u][idx];
            if cu == cv {
                // Intra-community: becomes a self-loop on super-node cu.
                *edge_map.entry((cu, cu)).or_insert(0.0) += w;
            } else {
                let key = if cu < cv { (cu, cv) } else { (cv, cu) };
                *edge_map.entry(key).or_insert(0.0) += w;
            }
        }

        // Source self-loops propagate to the aggregate super-node unchanged.
        if graph.self_loops[u] > 0.0 {
            *edge_map.entry((cu, cu)).or_insert(0.0) += graph.self_loops[u];
        }
    }

    let edges_w: Vec<(usize, usize, f64)> =
        edge_map.iter().map(|(&(u, v), &w)| (u, v, w)).collect();
    let agg_graph = LeidenGraph::from_edges_weighted(num_comms, &edges_w);

    AggregateResult {
        graph: agg_graph,
        membership,
    }
}

pub(crate) fn flatten_partition(
    agg_partition: &[usize],
    membership: &[Vec<usize>],
    original_n: usize,
) -> Vec<usize> {
    let mut assignment = vec![0usize; original_n];
    for (super_node, members) in membership.iter().enumerate() {
        let comm = agg_partition[super_node];
        for &orig in members {
            assignment[orig] = comm;
        }
    }
    assignment
}

pub(crate) fn map_to_aggregate_init(
    coarse_partition: &[usize],
    refined_partition: &[usize],
    num_super_nodes: usize,
) -> Vec<usize> {
    let mut agg_init = vec![0usize; num_super_nodes];
    for (node, &super_node) in refined_partition.iter().enumerate() {
        agg_init[super_node] = coarse_partition[node];
    }
    agg_init
}
