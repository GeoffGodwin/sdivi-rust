//! Network aggregation phase of the Leiden algorithm.
//!
//! Collapses the current graph into an aggregate graph where each community in
//! the refined partition becomes a single super-node.  Edge weights between
//! super-nodes equal the number of edges between the corresponding communities.
//! Self-loops (intra-community edges) are excluded.

use super::graph::LeidenGraph;

/// Result of a single aggregation step.
pub(crate) struct AggregateResult {
    /// The aggregated (smaller) graph.
    pub graph: LeidenGraph,
    /// `membership[super_node]` = list of original node indices in that super-node.
    pub membership: Vec<Vec<usize>>,
}

/// Builds an aggregate graph from `graph` and a `refined_partition`.
///
/// Returns the aggregate graph and the membership mapping.
///
/// If the refined partition has the same number of communities as nodes
/// (no merging happened), the aggregate graph equals the original and the
/// caller should stop recursing.
pub(crate) fn aggregate_network(
    graph: &LeidenGraph,
    refined_partition: &[usize],
) -> AggregateResult {
    let n = graph.n;
    let num_comms = refined_partition.iter().copied().max().map(|m| m + 1).unwrap_or(0);

    // Collect original nodes into each super-node.
    let mut membership: Vec<Vec<usize>> = vec![vec![]; num_comms];
    for (node, &comm) in refined_partition.iter().enumerate() {
        membership[comm].push(node);
    }

    // Build aggregate edge set: (comm_u, comm_v) → weight.
    // Use a BTreeMap for determinism.
    let mut edge_map: std::collections::BTreeMap<(usize, usize), f64> =
        std::collections::BTreeMap::new();

    for u in 0..n {
        let cu = refined_partition[u];
        for &v in &graph.adj[u] {
            let cv = refined_partition[v];
            if cu < cv {
                *edge_map.entry((cu, cv)).or_insert(0.0) += 1.0;
            } else if cu > cv {
                *edge_map.entry((cv, cu)).or_insert(0.0) += 1.0;
            }
            // cu == cv: intra-community edge, skip (self-loop in aggregate).
        }
    }

    // Build edges list (each undirected edge appears once).
    let edges: Vec<(usize, usize)> = edge_map.keys().copied().collect();
    let agg_graph = LeidenGraph::from_edges(num_comms, &edges);

    AggregateResult { graph: agg_graph, membership }
}

/// Maps the coarse partition over the aggregate graph back to original nodes.
///
/// `agg_partition[super_node]` = community in the aggregate result.
/// `membership[super_node]` = original nodes in that super-node.
///
/// Returns a flat `assignment[original_node]` = final community.
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

/// Maps the pre-recursion coarse partition to initial aggregate-level
/// community assignments.
///
/// `coarse_partition[orig_node]` = coarse community.
/// `refined_partition[orig_node]` = refined community (= aggregate super-node).
///
/// Returns `agg_init[super_node]` = the coarse community the super-node
/// belongs to (used to prime the aggregate Leiden run).
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
