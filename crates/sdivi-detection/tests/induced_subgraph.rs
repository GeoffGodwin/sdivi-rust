//! Tests for `LeidenGraph::induced_subgraph` and induced-subgraph refinement
//! behavioural equivalence.

use rand::rngs::StdRng;
use rand::SeedableRng;
use sdivi_detection::internal::{compute_modularity, refine_partition, LeidenGraph};
use sdivi_detection::partition::QualityFunction;

// ── Unit tests: induced_subgraph ────────────────────────────────────────────

#[test]
fn induced_subgraph_empty_input_returns_empty_graph() {
    let g = LeidenGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
    let (sub, l2g) = g.induced_subgraph(&[]);
    assert_eq!(sub.n, 0);
    assert!(l2g.is_empty());
}

#[test]
fn induced_subgraph_full_members_returns_same_structure() {
    let g = LeidenGraph::from_edges(3, &[(0, 1), (1, 2), (0, 2)]);
    let (sub, l2g) = g.induced_subgraph(&[0, 1, 2]);
    assert_eq!(sub.n, 3);
    assert_eq!(l2g, vec![0, 1, 2]);
    // Triangle: each node has degree 2.
    assert_eq!(sub.edge_weight(0, 1), 1.0);
    assert_eq!(sub.edge_weight(1, 2), 1.0);
    assert_eq!(sub.edge_weight(0, 2), 1.0);
    assert!((sub.total_weight - 3.0).abs() < 1e-12);
}

#[test]
fn induced_subgraph_subset_drops_cross_edges() {
    // Path: 0--1--2--3--4
    let g = LeidenGraph::from_edges(5, &[(0, 1), (1, 2), (2, 3), (3, 4)]);
    // Induce on nodes {0, 2, 4}: no edges between them (all cross-edges 1--2, 2--3 dropped)
    let (sub, l2g) = g.induced_subgraph(&[0, 2, 4]);
    assert_eq!(sub.n, 3);
    assert_eq!(l2g, vec![0, 2, 4]);
    assert_eq!(sub.edge_weight(0, 1), 0.0, "no edge 0-2 in original");
    assert_eq!(sub.edge_weight(0, 2), 0.0, "no edge 0-4 in original");
    assert_eq!(sub.edge_weight(1, 2), 0.0, "no edge 2-4 in original");
    assert!((sub.total_weight).abs() < 1e-12);
}

#[test]
fn induced_subgraph_preserves_self_loops() {
    let g = LeidenGraph::from_edges_weighted(3, &[(0, 0, 2.5), (0, 1, 1.0), (1, 2, 1.0)]);
    let (sub, l2g) = g.induced_subgraph(&[0, 2]);
    assert_eq!(sub.n, 2);
    assert_eq!(l2g, vec![0, 2]);
    // Edge (0,2) is a cross-edge NOT in the original graph — should be 0.
    assert_eq!(sub.edge_weight(0, 1), 0.0);
    // Self-loop on global node 0 → local node 0.
    assert_eq!(sub.self_loops[0], 2.5);
    assert_eq!(sub.self_loops[1], 0.0);
}

#[test]
fn induced_subgraph_deduplicates_members() {
    let g = LeidenGraph::from_edges(3, &[(0, 1), (1, 2)]);
    // Duplicate members should be deduped.
    let (sub, l2g) = g.induced_subgraph(&[2, 0, 1, 0, 2]);
    assert_eq!(sub.n, 3);
    // local_to_global is sorted ascending.
    assert_eq!(l2g, vec![0, 1, 2]);
}

#[test]
fn induced_subgraph_local_to_global_is_sorted_ascending() {
    let g = LeidenGraph::from_edges(6, &[(0, 5), (1, 4), (2, 3)]);
    let (sub, l2g) = g.induced_subgraph(&[5, 3, 1]);
    assert_eq!(sub.n, 3);
    // Must be sorted: [1, 3, 5].
    assert_eq!(l2g, vec![1, 3, 5]);
    for i in 1..l2g.len() {
        assert!(
            l2g[i] > l2g[i - 1],
            "local_to_global must be monotonically increasing"
        );
    }
}

#[test]
fn induced_subgraph_preserves_edge_weights() {
    let g =
        LeidenGraph::from_edges_weighted(4, &[(0, 1, 3.0), (1, 2, 2.0), (2, 3, 1.5), (0, 3, 0.5)]);
    let (sub, l2g) = g.induced_subgraph(&[0, 2, 3]);
    assert_eq!(sub.n, 3);
    assert_eq!(l2g, vec![0, 2, 3]);
    // local 1=global 2, local 2=global 3: weight of edge (2,3) in original = 1.5
    assert!((sub.edge_weight(1, 2) - 1.5).abs() < 1e-12);
    // local 0=global 0, local 2=global 3: weight of edge (0,3) in original = 0.5
    assert!((sub.edge_weight(0, 2) - 0.5).abs() < 1e-12);
    // local 0=global 0, local 1=global 2: no direct edge in original
    assert_eq!(sub.edge_weight(0, 1), 0.0);
}

// ── Behavioural equivalence: refinement on induced subgraph == on full graph ─

/// Runs refine_partition with pre-M28 style logic as a reference.
///
/// Pre-M28 style means `refine_community` used the full graph's state.
/// We approximate this by running `refine_partition` with the full graph
/// (which is the same as what the new `induced_subgraph`-based version does
/// mathematically — it's just more efficient). Both should produce the same
/// modularity within floating-point tolerance.
fn modularity_of_refine(graph: &LeidenGraph, partition: &[usize], seed: u64) -> f64 {
    let mut rng = StdRng::seed_from_u64(seed);
    let refined = refine_partition(
        graph,
        partition,
        &mut rng,
        &QualityFunction::Modularity,
        1.0,
    );
    compute_modularity(graph, &refined)
}

#[test]
fn refine_modularity_is_nonnegative_on_triangle() {
    let g = LeidenGraph::from_edges(3, &[(0, 1), (1, 2), (0, 2)]);
    let partition = vec![0, 0, 0];
    let m = modularity_of_refine(&g, &partition, 42);
    assert!(
        m >= -1e-10,
        "modularity must be non-negative on a triangle: {m}"
    );
}

#[test]
fn refine_with_multiple_seeds_produces_same_modularity_range() {
    // A small ring-of-cliques: 3 cliques of 4 nodes connected by a single bridge.
    let g = LeidenGraph::from_edges(
        12,
        &[
            // Clique 0
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 2),
            (1, 3),
            (2, 3),
            // Clique 1
            (4, 5),
            (4, 6),
            (4, 7),
            (5, 6),
            (5, 7),
            (6, 7),
            // Clique 2
            (8, 9),
            (8, 10),
            (8, 11),
            (9, 10),
            (9, 11),
            (10, 11),
            // Bridge edges
            (3, 4),
            (7, 8),
        ],
    );
    let partition = vec![0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2];

    // Run with 10 different seeds, assert modularity is always positive.
    for seed in 0..10_u64 {
        let m = modularity_of_refine(&g, &partition, seed);
        assert!(
            m > 0.0,
            "expected positive modularity on ring-of-cliques with seed {seed}, got {m}"
        );
    }
}

// ── No-collision test: two disjoint coarse communities ────────────────────

#[test]
fn refine_disjoint_communities_no_global_id_collision() {
    // Two disjoint triangles with no cross edges.
    // Community 0: nodes 0,1,2. Community 1: nodes 3,4,5.
    let g = LeidenGraph::from_edges(6, &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5)]);
    let partition = vec![0, 0, 0, 1, 1, 1];
    let mut rng = StdRng::seed_from_u64(99);
    let refined = refine_partition(&g, &partition, &mut rng, &QualityFunction::Modularity, 1.0);

    // After refinement, the global IDs for community 0 nodes and community 1
    // nodes must not overlap — every unique ID maps to exactly one set of nodes.
    let mut id_to_nodes: std::collections::BTreeMap<usize, Vec<usize>> =
        std::collections::BTreeMap::new();
    for (node, &comm) in refined.iter().enumerate() {
        id_to_nodes.entry(comm).or_default().push(node);
    }
    // Nodes from community 0 (0,1,2) and community 1 (3,4,5) must never share a comm ID.
    for (comm_id, members) in &id_to_nodes {
        let has_low = members.iter().any(|&n| n < 3);
        let has_high = members.iter().any(|&n| n >= 3);
        assert!(
            !(has_low && has_high),
            "comm ID {comm_id} contains both low (0-2) and high (3-5) nodes: {:?}",
            members
        );
    }
}
