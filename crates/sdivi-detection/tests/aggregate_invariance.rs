//! Tests for the modularity-invariance-under-aggregation property.
//!
//! Verifies that `compute_modularity(graph, partition)` equals
//! `compute_modularity(aggregate(graph, partition), identity)` within 1e-9.
//! This is the mathematical property the Leiden aggregation step relies on.

use proptest::prelude::*;
use sdivi_detection::internal::{aggregate_network, compute_modularity, LeidenGraph};

// ── Hand-derived test cases ──────────────────────────────────────────────────

/// Two isolated cliques — aggregate must carry self-loops, no cross-edges.
///
/// Original: nodes {0,1} and {2,3}, internal edges weight 1.0, no cross-edges.
/// Partition: [0,0,1,1].
/// Expected aggregate: 2 super-nodes, `self_loops = [1.0, 1.0]`, `total_weight = 2.0`.
#[test]
fn aggregate_two_cliques_no_cross_edges() {
    let g = LeidenGraph::from_edges_weighted(4, &[(0, 1, 1.0), (2, 3, 1.0)]);
    let partition = [0usize, 0, 1, 1];
    let agg = aggregate_network(&g, &partition);

    assert_eq!(agg.graph.n, 2, "two communities → two super-nodes");
    assert!(
        agg.graph.adj[0].is_empty() && agg.graph.adj[1].is_empty(),
        "no cross-edges between communities"
    );
    assert!(
        (agg.graph.self_loops[0] - 1.0).abs() < 1e-12,
        "intra-community edge becomes self-loop on super-node 0"
    );
    assert!(
        (agg.graph.self_loops[1] - 1.0).abs() < 1e-12,
        "intra-community edge becomes self-loop on super-node 1"
    );
    assert!(
        (agg.graph.total_weight - 2.0).abs() < 1e-12,
        "total_weight preserved"
    );
    // Degree: no cross-edges, self-loop weight 1.0 → degree = 2×1.0 = 2.0.
    assert!((agg.graph.degree[0] - 2.0).abs() < 1e-12);
    assert!((agg.graph.degree[1] - 2.0).abs() < 1e-12);
}

/// Two cliques plus a cross-edge — aggregate has both self-loops and a cross-edge.
///
/// Original: nodes {0,1} and {2,3}, internal edges weight 1.0, cross-edge (1,2,1.0).
/// Partition: [0,0,1,1].
/// Expected aggregate: `self_loops = [1.0, 1.0]`, one cross-edge (0,1) weight 1.0,
/// `total_weight = 3.0`.
#[test]
fn aggregate_two_cliques_with_cross_edge() {
    let g = LeidenGraph::from_edges_weighted(4, &[(0, 1, 1.0), (2, 3, 1.0), (1, 2, 1.0)]);
    let partition = [0usize, 0, 1, 1];
    let agg = aggregate_network(&g, &partition);

    assert_eq!(agg.graph.n, 2);
    // Cross-edge between communities must appear exactly once (not doubled).
    assert_eq!(
        agg.graph.adj[0],
        vec![1],
        "super-node 0 has one cross-edge to 1"
    );
    assert!(
        (agg.graph.edge_weights[0][0] - 1.0).abs() < 1e-12,
        "cross-edge weight 1.0"
    );
    assert!((agg.graph.self_loops[0] - 1.0).abs() < 1e-12);
    assert!((agg.graph.self_loops[1] - 1.0).abs() < 1e-12);
    assert!(
        (agg.graph.total_weight - 3.0).abs() < 1e-12,
        "1 cross + 1 + 1 self = 3.0"
    );
}

/// Modularity invariance for two isolated cliques.
///
/// Q(original, [0,0,1,1]) must equal Q(aggregate, [0,1]).
#[test]
fn modularity_invariance_two_cliques_no_cross() {
    let g = LeidenGraph::from_edges_weighted(4, &[(0, 1, 1.0), (2, 3, 1.0)]);
    let partition = [0usize, 0, 1, 1];

    let q_orig = compute_modularity(&g, &partition);

    let agg = aggregate_network(&g, &partition);
    let agg_partition: Vec<usize> = (0..agg.graph.n).collect();
    let q_agg = compute_modularity(&agg.graph, &agg_partition);

    assert!(
        (q_orig - q_agg).abs() < 1e-9,
        "modularity invariance: q_orig={q_orig:.12} q_agg={q_agg:.12}"
    );
}

/// Modularity invariance for two cliques connected by a cross-edge.
#[test]
fn modularity_invariance_two_cliques_with_cross() {
    let g = LeidenGraph::from_edges_weighted(4, &[(0, 1, 1.0), (2, 3, 1.0), (1, 2, 1.0)]);
    let partition = [0usize, 0, 1, 1];

    let q_orig = compute_modularity(&g, &partition);

    let agg = aggregate_network(&g, &partition);
    let agg_partition: Vec<usize> = (0..agg.graph.n).collect();
    let q_agg = compute_modularity(&agg.graph, &agg_partition);

    assert!(
        (q_orig - q_agg).abs() < 1e-9,
        "modularity invariance: q_orig={q_orig:.12} q_agg={q_agg:.12}"
    );
}

// ── Property test ────────────────────────────────────────────────────────────

fn check_modularity_invariance(n: usize, edges: &[(usize, usize, f64)], partition: &[usize]) {
    let g = LeidenGraph::from_edges_weighted(n, edges);
    if g.total_weight < 1e-12 {
        return; // Q is trivially 0 on both sides for empty graphs.
    }
    let q_orig = compute_modularity(&g, partition);
    let agg = aggregate_network(&g, partition);
    let agg_partition: Vec<usize> = (0..agg.graph.n).collect();
    let q_agg = compute_modularity(&agg.graph, &agg_partition);
    assert!(
        (q_orig - q_agg).abs() < 1e-9,
        "n={n} modularity not invariant under aggregation: q_orig={q_orig:.12} q_agg={q_agg:.12}"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// For any graph and any partition, modularity is preserved under aggregation.
    ///
    /// Generates a random `n`-node graph and a random partition into up to 3
    /// communities, then asserts Q(original, partition) ≈ Q(aggregate, identity).
    #[test]
    fn prop_aggregate_modularity_invariance(
        n in 2usize..=8usize,
        // Raw edge indices; clamped to [0, n) in the body.
        raw_edges in prop::collection::vec(
            (0usize..8usize, 0usize..8usize, 0.1f64..=3.0f64),
            0..12,
        ),
        // Raw partition values; clamped to [0, 3) and truncated to n.
        partition_raw in prop::collection::vec(0usize..=2usize, 8),
    ) {
        let edges: Vec<(usize, usize, f64)> = raw_edges
            .into_iter()
            .map(|(u, v, w)| (u % n, v % n, w))
            .collect();
        // Take first n values; since partition_raw has exactly 8 elements and n ≤ 8 this is always valid.
        let partition: Vec<usize> = partition_raw[..n].to_vec();
        check_modularity_invariance(n, &edges, &partition);
    }
}
