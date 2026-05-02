//! Network aggregation phase of the Leiden algorithm.

use super::graph::LeidenGraph;

pub(crate) struct AggregateResult {
    pub graph: LeidenGraph,
    pub membership: Vec<Vec<usize>>,
}

pub(crate) fn aggregate_network(
    graph: &LeidenGraph,
    refined_partition: &[usize],
) -> AggregateResult {
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

    let mut edge_map: std::collections::BTreeMap<(usize, usize), f64> =
        std::collections::BTreeMap::new();

    for u in 0..n {
        let cu = refined_partition[u];
        for (idx, &v) in graph.adj[u].iter().enumerate() {
            let cv = refined_partition[v];
            let w = graph.edge_weights[u][idx];
            match cu.cmp(&cv) {
                std::cmp::Ordering::Less => *edge_map.entry((cu, cv)).or_insert(0.0) += w,
                std::cmp::Ordering::Greater => *edge_map.entry((cv, cu)).or_insert(0.0) += w,
                std::cmp::Ordering::Equal => {}
            }
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
