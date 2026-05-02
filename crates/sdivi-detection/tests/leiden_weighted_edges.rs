//! Tests that weighted Leiden produces a different/better partition.

use sdivi_detection::{run_leiden, run_leiden_with_weights, LeidenConfig};
use sdivi_graph::dependency_graph::build_dependency_graph_from_edges;
use std::collections::BTreeMap;

/// Two triangles connected by a single bridge edge.
/// Unweighted: may merge; weighted (bridge boosted): should separate.
fn two_triangles_dg() -> sdivi_graph::dependency_graph::DependencyGraph {
    build_dependency_graph_from_edges(
        &(0..6).map(|i| format!("node{i}")).collect::<Vec<_>>(),
        &[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (0, 3)], // bridge at (0,3)
    )
}

#[test]
fn weighted_produces_different_or_better_partition() {
    let dg = two_triangles_dg();
    let cfg = LeidenConfig::default();

    let unweighted = run_leiden(&dg, &cfg, None);

    // Boost intra-triangle edges to encourage separation.
    let mut weight_map = BTreeMap::new();
    for &(u, v) in &[(0, 1), (1, 2), (0, 2), (3, 4), (4, 5), (3, 5)] {
        weight_map.insert((u, v), 3.0); // heavy intra-community
    }
    // Bridge (0,3) stays at default weight (not in map → 1.0).

    let weighted = run_leiden_with_weights(&dg, &cfg, None, &weight_map);

    // Weighted modularity must not be less than unweighted.
    assert!(
        weighted.modularity >= unweighted.modularity - 1e-9,
        "weighted modularity {:.6} < unweighted {:.6}",
        weighted.modularity,
        unweighted.modularity
    );
}
