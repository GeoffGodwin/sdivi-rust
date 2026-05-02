//! Regression tests for the Leiden community-ID / node-index collision underflow bug.
//!
//! Background: before the M08 offset fix, `local_move_phase` passed a partition
//! vector with community IDs in 0..k directly to `ModularityState::from_assignment`.
//! When `remove_node(X)` was called it wrote `size[X] = 1` to the singleton slot,
//! using X's node index as the community ID.  If community X had multiple members
//! and X was processed early in the shuffle order, this reset `size[X]` to 1 while
//! real members were still assigned to community X.  Subsequent `remove_node` calls
//! for those members decremented `size[X]` below zero, causing a usize underflow
//! (panic in debug builds).
//!
//! Fix (M08): `local_move_phase` offsets all community IDs by n before building
//! `ModularityState`, so singleton slots 0..n never overlap with community slots
//! n..n+k.

use std::collections::BTreeMap;

use sdivi_detection::leiden::run_leiden;
use sdivi_detection::partition::{LeidenConfig, LeidenPartition};
use sdivi_graph::dependency_graph::build_dependency_graph_from_edges;

fn ring_graph(n: usize) -> sdivi_graph::DependencyGraph {
    let nodes: Vec<String> = (0..n).map(|i| format!("src/n{i}.rs")).collect();
    let edges: Vec<(usize, usize)> = (0..n).map(|i| (i, (i + 1) % n)).collect();
    build_dependency_graph_from_edges(&nodes, &edges)
}

fn clique_graph(n: usize) -> sdivi_graph::DependencyGraph {
    let nodes: Vec<String> = (0..n).map(|i| format!("src/n{i}.rs")).collect();
    let edges: Vec<(usize, usize)> = (0..n)
        .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
        .collect();
    build_dependency_graph_from_edges(&nodes, &edges)
}

// ── Reviewer-specified regression: n=4, community IDs equal to node indices ──

/// Reviewer-described scenario: n=4, warm-start partition=[0,1,2,3] so
/// community 0 == node 0, community 1 == node 1, etc.
///
/// Each community is a singleton but community IDs collide with node indices.
/// The offset fix must not corrupt the singleton-slot logic in `remove_node`.
#[test]
fn leiden_singleton_partition_with_ids_equal_to_node_indices_completes() {
    let dg = ring_graph(4);
    let cfg = LeidenConfig {
        seed: 42,
        ..LeidenConfig::default()
    };

    // Community i = node i for all i — IDs identical to node indices.
    let warm_start = LeidenPartition {
        assignments: BTreeMap::from([(0, 0), (1, 1), (2, 2), (3, 3)]),
        stability: BTreeMap::from([(0, 1.0), (1, 1.0), (2, 1.0), (3, 1.0)]),
        modularity: 0.0,
        seed: 42,
    };

    let partition = run_leiden(&dg, &cfg, Some(&warm_start));

    assert_eq!(
        partition.assignments.len(),
        4,
        "all 4 nodes must be assigned"
    );
    assert!(
        partition.community_count() >= 1,
        "at least one community must exist"
    );
    // All community IDs must be in [0, community_count).
    let k = partition.community_count();
    for (&node, &comm) in &partition.assignments {
        assert!(
            comm < k,
            "node {node} assigned to community {comm} which is out of range [0, {k})"
        );
    }
}

// ── Primary regression: multi-node community with ID == node index ────────────

/// The actual underflow trigger: three nodes share community 0 (ID == node 0).
/// Without the offset fix, if node 0 is shuffled before nodes 1 and 2, the
/// `remove_node(0)` call resets `size[0]` to 1 (phantom), then the two
/// subsequent removes from community 0 decrement `size[0]` to -1 (usize
/// overflow / panic).
#[test]
fn leiden_three_nodes_in_community_zero_no_underflow() {
    let dg = clique_graph(4);
    let cfg = LeidenConfig {
        seed: 42,
        ..LeidenConfig::default()
    };

    // Nodes 0, 1, 2 in community 0; node 3 in community 1.
    // Community ID 0 == node index 0 with 3 members — the exact underflow trigger.
    let warm_start = LeidenPartition {
        assignments: BTreeMap::from([(0, 0), (1, 0), (2, 0), (3, 1)]),
        stability: BTreeMap::from([(0, 0.8), (1, 0.5)]),
        modularity: 0.3,
        seed: 42,
    };

    let partition = run_leiden(&dg, &cfg, Some(&warm_start));

    assert_eq!(
        partition.assignments.len(),
        4,
        "all 4 nodes must be assigned"
    );
    assert!(partition.community_count() >= 1);
    let k = partition.community_count();
    for (&node, &comm) in &partition.assignments {
        assert!(
            comm < k,
            "node {node}: community {comm} out of range [0, {k})"
        );
    }
}

/// Worst-case: ALL nodes in community 0.  With n members and no offset fix,
/// removing node 0 at any position before the last two guarantees underflow.
#[test]
fn leiden_all_nodes_in_community_zero_no_underflow() {
    let n = 8usize;
    let dg = clique_graph(n);
    let cfg = LeidenConfig {
        seed: 42,
        ..LeidenConfig::default()
    };

    let assignments: BTreeMap<usize, usize> = (0..n).map(|i| (i, 0)).collect();
    let warm_start = LeidenPartition {
        assignments,
        stability: BTreeMap::from([(0, 1.0)]),
        modularity: 0.0,
        seed: 42,
    };

    let partition = run_leiden(&dg, &cfg, Some(&warm_start));

    assert_eq!(
        partition.assignments.len(),
        n,
        "all {n} nodes must be assigned"
    );
    assert!(partition.community_count() >= 1);
}

/// Regression for multi-iteration Leiden: after the first local_move_phase and
/// renumber, subsequent iterations receive a partition in 0..k which again has
/// community IDs potentially equal to node indices.  The fix must hold across
/// all iterations, not just the first.
#[test]
fn leiden_multi_iteration_no_underflow_on_two_cliques() {
    // Two 5-node cliques connected by a single bridge.  The algorithm needs
    // multiple iterations to converge, so `local_move_phase` is called several
    // times with renumbered partitions — exercising the offset fix repeatedly.
    let nodes: Vec<String> = (0..10).map(|i| format!("src/n{i}.rs")).collect();
    let mut edges: Vec<(usize, usize)> = Vec::new();
    // Clique A: nodes 0..4
    for i in 0..5 {
        for j in i + 1..5 {
            edges.push((i, j));
        }
    }
    // Clique B: nodes 5..9
    for i in 5..10 {
        for j in i + 1..10 {
            edges.push((i, j));
        }
    }
    // Bridge: 4 → 5
    edges.push((4, 5));

    let dg = build_dependency_graph_from_edges(&nodes, &edges);

    // Warm start with community IDs matching node indices.
    let warm_start = LeidenPartition {
        assignments: BTreeMap::from([
            (0, 0),
            (1, 0),
            (2, 0),
            (3, 0),
            (4, 0),
            (5, 5),
            (6, 5),
            (7, 5),
            (8, 5),
            (9, 5),
        ]),
        stability: BTreeMap::from([(0, 0.9), (5, 0.9)]),
        modularity: 0.4,
        seed: 42,
    };

    let cfg = LeidenConfig {
        seed: 42,
        max_iterations: 10,
        ..LeidenConfig::default()
    };
    let partition = run_leiden(&dg, &cfg, Some(&warm_start));

    assert_eq!(
        partition.assignments.len(),
        10,
        "all 10 nodes must be assigned"
    );
    assert!(
        partition.community_count() >= 1,
        "at least one community must exist"
    );
    let k = partition.community_count();
    for (&node, &comm) in &partition.assignments {
        assert!(
            comm < k,
            "node {node}: community {comm} out of range [0, {k})"
        );
    }
}

// ── Cold-start sanity: ID collision in initial singletons ────────────────────

/// Cold-start produces singletons [0, 1, ..., n-1] after renumber — every
/// community ID equals its node index.  Verifies the offset fix holds for the
/// baseline (no warm start) path as well.
#[test]
fn leiden_cold_start_ring_no_panic() {
    let dg = ring_graph(5);
    let cfg = LeidenConfig {
        seed: 42,
        ..LeidenConfig::default()
    };

    let partition = run_leiden(&dg, &cfg, None);

    assert_eq!(partition.assignments.len(), 5);
    assert!(partition.community_count() >= 1);
}
