//! End-to-end regression tests for the M17 Leiden bug fix.
//!
//! Verifies that `run_leiden` correctly identifies multiple stable communities
//! after two correctness bugs in `aggregate_network` were fixed:
//!
//! 1. Intra-community cross-edges were silently dropped instead of becoming
//!    self-loops on the aggregate super-node (causing the aggregate graph to
//!    have zero intra-community weight, making the "merge all" partition look
//!    optimal by default).
//! 2. Inter-community edges were double-counted (upper-triangle fix).
//!
//! Together these caused every graph to collapse to a single community after
//! one aggregation step.
//!
//! # Manual verification item
//!
//! `compute_stability` with self-loops present cannot be directly exercised from
//! integration tests: it is `pub(crate)` and not re-exported via `internal`.
//! The reviewer noted it as low-priority given the inert call site (the original
//! `LeidenGraph` is always built from a `DependencyGraph`, which has no
//! self-loops). Direct testing would require adding `compute_stability` to the
//! `internal` module — deferred to a future milestone.

use sdivi_detection::leiden::run_leiden;
use sdivi_detection::partition::LeidenConfig;
use sdivi_graph::dependency_graph::build_dependency_graph_from_edges;

// ── Primary regression: two disconnected cliques → two communities ────────────

/// Two disconnected triangle cliques must produce exactly two communities.
///
/// Pre-fix: `aggregate_network` dropped all intra-community edges and
/// double-counted inter-community edges, making the aggregate graph weightless
/// and causing collapse to a single community.
///
/// Post-fix: intra-community edges become self-loops on the super-node,
/// preserving the modularity signal that keeps the two cliques separated.
#[test]
fn two_disconnected_triangle_cliques_produce_two_communities() {
    let paths: Vec<String> = (0..6).map(|i| format!("src/node{i}.rs")).collect();
    // Clique 1: nodes 0,1,2; clique 2: nodes 3,4,5; no cross-edges.
    let edges = vec![
        (0, 1),
        (1, 2),
        (0, 2), // clique 1
        (3, 4),
        (4, 5),
        (3, 5), // clique 2
    ];
    let dg = build_dependency_graph_from_edges(&paths, &edges);
    let cfg = LeidenConfig::default();

    let partition = run_leiden(&dg, &cfg, None);

    assert_eq!(
        partition.community_count(),
        2,
        "two disconnected cliques must yield two communities; got assignments: {:?}",
        partition.communities()
    );

    // Nodes within each clique must share a community.
    let c0 = partition.community_of(0).expect("node 0 in partition");
    let c1 = partition.community_of(1).expect("node 1 in partition");
    let c2 = partition.community_of(2).expect("node 2 in partition");
    let c3 = partition.community_of(3).expect("node 3 in partition");
    let c4 = partition.community_of(4).expect("node 4 in partition");
    let c5 = partition.community_of(5).expect("node 5 in partition");

    assert_eq!(c0, c1, "nodes 0 and 1 must be in the same community");
    assert_eq!(c1, c2, "nodes 1 and 2 must be in the same community");

    assert_eq!(c3, c4, "nodes 3 and 4 must be in the same community");
    assert_eq!(c4, c5, "nodes 4 and 5 must be in the same community");

    // The two cliques must be in distinct communities.
    assert_ne!(
        c0, c3,
        "clique 1 (nodes 0-2) and clique 2 (nodes 3-5) must be in different communities"
    );
}

/// The two-clique partition must have positive modularity.
///
/// Q ≈ 0.5 for two balanced disconnected triangles (analytically derivable).
/// A collapsed single-community partition yields Q = 0.  A positive result
/// confirms that Leiden found the meaningful split, not the trivial merge.
#[test]
fn two_disconnected_triangle_cliques_have_positive_modularity() {
    let paths: Vec<String> = (0..6).map(|i| format!("src/node{i}.rs")).collect();
    let edges = vec![(0, 1), (1, 2), (0, 2), (3, 4), (4, 5), (3, 5)];
    let dg = build_dependency_graph_from_edges(&paths, &edges);
    let cfg = LeidenConfig::default();

    let partition = run_leiden(&dg, &cfg, None);

    assert!(
        partition.modularity > 0.0,
        "expected positive modularity for two-clique partition, got {}",
        partition.modularity
    );
}

// ── Degenerate cases: fix must not regress simple inputs ─────────────────────

/// An empty graph produces an empty partition (zero communities).
#[test]
fn empty_graph_produces_zero_communities() {
    let dg = build_dependency_graph_from_edges(&[], &[]);
    let cfg = LeidenConfig::default();

    let partition = run_leiden(&dg, &cfg, None);

    assert_eq!(partition.community_count(), 0);
}

/// A single isolated node (no edges) produces exactly one singleton community.
#[test]
fn single_node_produces_one_community() {
    let paths = vec!["src/alone.rs".to_string()];
    let dg = build_dependency_graph_from_edges(&paths, &[]);
    let cfg = LeidenConfig::default();

    let partition = run_leiden(&dg, &cfg, None);

    assert_eq!(partition.community_count(), 1);
    assert_eq!(partition.community_of(0), Some(0));
}

/// Two isolated nodes (no edges) produce exactly two singleton communities.
#[test]
fn two_isolated_nodes_produce_two_communities() {
    let paths = vec!["src/a.rs".to_string(), "src/b.rs".to_string()];
    let dg = build_dependency_graph_from_edges(&paths, &[]);
    let cfg = LeidenConfig::default();

    let partition = run_leiden(&dg, &cfg, None);

    assert_eq!(partition.community_count(), 2);
    let ca = partition.community_of(0).expect("node 0 in partition");
    let cb = partition.community_of(1).expect("node 1 in partition");
    assert_ne!(
        ca, cb,
        "two isolated nodes must be in different communities"
    );
}

/// Run is deterministic: same seed and graph always produce the same partition.
#[test]
fn run_leiden_is_deterministic() {
    let paths: Vec<String> = (0..6).map(|i| format!("src/node{i}.rs")).collect();
    let edges = vec![(0, 1), (1, 2), (0, 2), (3, 4), (4, 5), (3, 5)];
    let dg = build_dependency_graph_from_edges(&paths, &edges);
    let cfg = LeidenConfig::default();

    let p1 = run_leiden(&dg, &cfg, None);
    let p2 = run_leiden(&dg, &cfg, None);

    assert_eq!(p1, p2, "identical inputs must yield identical partitions");
}
