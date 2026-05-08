//! Performance regression test: sparse 1500-node graph must run in <2s debug.
//!
//! Pre-M28 this would run for minutes due to full-graph state allocation
//! per coarse community and near-identity recursion compression.

use rand::rngs::StdRng;
use rand::SeedableRng;
use sdivi_detection::leiden::run_leiden;
use sdivi_detection::partition::LeidenConfig;
use sdivi_graph::dependency_graph::build_dependency_graph_from_edges;
use std::time::Instant;

/// Builds a sparse random graph: 1500 nodes, ~600 edges (average degree < 1).
///
/// Uses a fixed seed so the test is reproducible. The graph is intentionally
/// sparse to exercise the pre-M28 worst case: many small coarse communities,
/// each triggering a full-graph refinement setup.
fn sparse_graph_fixture() -> (Vec<String>, Vec<(usize, usize)>) {
    use rand::Rng;
    let n = 1500_usize;
    let mut rng = StdRng::seed_from_u64(20280628);
    let node_paths: Vec<String> = (0..n).map(|i| format!("src/module_{i}.rs")).collect();

    // Build ~600 edges by randomly pairing nodes (some may be duplicates which are fine).
    let mut edges: Vec<(usize, usize)> = Vec::with_capacity(600);
    for _ in 0..600 {
        let u = rng.gen_range(0..n);
        let v = rng.gen_range(0..n);
        if u != v {
            edges.push((u, v));
        }
    }
    // Also add a handful of small cliques to give Leiden something to cluster.
    for base in [0, 300, 600, 900, 1200] {
        for i in 0..5 {
            for j in (i + 1)..5 {
                edges.push((base + i, base + j));
            }
        }
    }
    (node_paths, edges)
}

#[test]
fn sparse_1500_node_graph_runs_within_time_limit() {
    let (node_paths, edges) = sparse_graph_fixture();
    let dg = build_dependency_graph_from_edges(&node_paths, &edges);
    let cfg = LeidenConfig::default();

    let start = Instant::now();
    let partition = run_leiden(&dg, &cfg, None);
    let elapsed = start.elapsed();

    // Sanity: partition covers all nodes.
    assert_eq!(
        partition.assignments.len(),
        1500,
        "partition must cover all 1500 nodes"
    );

    if elapsed.as_secs_f64() > 5.0 {
        panic!(
            "sparse 1500-node Leiden took {:.2}s — expected <2s post-M28, hard limit 5s",
            elapsed.as_secs_f64()
        );
    }
    // Soft assertion: log a warning if it's over 2s but under the 5s hard limit.
    if elapsed.as_secs_f64() > 2.0 {
        eprintln!(
            "WARNING: sparse 1500-node Leiden took {:.2}s (goal: <2s; hard limit: 5s)",
            elapsed.as_secs_f64()
        );
    }
}
