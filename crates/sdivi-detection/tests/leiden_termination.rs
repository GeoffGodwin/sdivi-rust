//! Full-Leiden integration and M49.1 hang-regression tests.
//!
//! Contains tests that exercise `run_leiden` end-to-end, including the ignored
//! regression guard for the non-termination bug captured in M49.1.

use sdivi_detection::leiden::run_leiden;
use sdivi_detection::partition::LeidenConfig;
use sdivi_graph::dependency_graph::build_dependency_graph_from_edges;

// ── Full Leiden quality ───────────────────────────────────────────────────────

/// Full Leiden on a ring-of-3-cliques must produce positive modularity,
/// confirming that the corrected refinement no longer collapses all nodes.
#[test]
fn leiden_with_corrected_refine_gives_positive_modularity() {
    use sdivi_graph::dependency_graph::build_dependency_graph;
    use sdivi_parsing::feature_record::FeatureRecord;
    use std::path::PathBuf;

    let make = |p: &str, imports: &[&str]| FeatureRecord {
        path: PathBuf::from(p),
        language: "rust".into(),
        imports: imports.iter().map(|s| s.to_string()).collect(),
        exports: vec![],
        signatures: vec![],
        pattern_hints: vec![],
    };

    let records = vec![
        make("src/a0.rs", &["crate::a1", "crate::a2", "crate::a3"]),
        make("src/a1.rs", &["crate::a0", "crate::a2", "crate::a3"]),
        make("src/a2.rs", &["crate::a0", "crate::a1", "crate::a3"]),
        make(
            "src/a3.rs",
            &["crate::a0", "crate::a1", "crate::a2", "crate::b0"],
        ),
        make(
            "src/b0.rs",
            &["crate::b1", "crate::b2", "crate::b3", "crate::a3"],
        ),
        make("src/b1.rs", &["crate::b0", "crate::b2", "crate::b3"]),
        make("src/b2.rs", &["crate::b0", "crate::b1", "crate::b3"]),
        make("src/b3.rs", &["crate::b0", "crate::b1", "crate::b2"]),
    ];
    let dg = build_dependency_graph(&records);
    let cfg = LeidenConfig {
        seed: 42,
        ..LeidenConfig::default()
    };
    let p = run_leiden(&dg, &cfg, None);

    assert!(
        p.modularity > 0.1,
        "expected positive modularity (got {})",
        p.modularity
    );
    assert!(
        p.community_count() >= 2,
        "expected ≥2 communities (got {})",
        p.community_count()
    );
}

// ── M49.1 regression: captured minimal hanging case ───────────────────────────

/// Guard for the minimal `(n, edges, seed)` that causes `run_leiden` to hang.
///
/// Captured by `prop_refine_modularity_does_not_decrease` (with fork+timeout
/// enabled in M49.1) minimizing a timed-out case.  M49.2 restructured
/// `leiden_recursive` to descend into the aggregate at most once per level,
/// eliminating the O(max_iter^depth) blowup that caused this case to hang.
///
/// Minimal case: n=6, K_{1,5} star (node 3 connected to all others), seed=0.
/// Edges: [(3,1),(3,4),(3,2),(3,0),(3,5)].
///
/// The proptest regression is stored in `refinement.proptest-regressions`:
/// `cc 7bcb943d7e48407056966a9ca32d5f7f276d3d9e694db514ac5de978d843f27e`
#[test]
fn leiden_termination_regression_star_n6_seed0() {
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    // K_{1,5} star: node 3 is the hub; all edges connect 3 to its neighbours.
    // This is the minimal case proptest found that causes run_leiden to hang.
    let n = 6usize;
    let edges = vec![(3usize, 1usize), (3, 4), (3, 2), (3, 0), (3, 5)];
    let seed = 0u64;

    let node_paths: Vec<String> = (0..n).map(|i| format!("n{i}.rs")).collect();
    let dg = build_dependency_graph_from_edges(&node_paths, &edges);
    let cfg = LeidenConfig {
        seed,
        ..LeidenConfig::default()
    };

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(run_leiden(&dg, &cfg, None));
    });

    rx.recv_timeout(Duration::from_secs(30))
        .expect("run_leiden hung on K_{1,5} star (n=6, seed=0); M49.2 must fix this");
}

// ── M49.2: brute-force termination sweep ─────────────────────────────────────

/// Sweeps known-pathological and representative small graphs over all seeds
/// in `0..=255`, asserting `run_leiden` returns a valid partition on every case.
///
/// With the pre-M49.2 implementation, K_{1,5} with seed=0 would hang for
/// minutes; with the restructured `leiden_recursive` every case completes
/// in microseconds.
#[test]
fn termination_sweep_known_graphs_all_seeds() {
    let graphs: &[(&str, usize, &[(usize, usize)])] = &[
        // K_{1,5} star — the M49.1 captured minimal hang case
        ("star-K1-5", 6, &[(3, 1), (3, 4), (3, 2), (3, 0), (3, 5)]),
        // K4 complete graph (all pairs connected)
        ("k4", 4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]),
        // Path graph n=6
        ("path-n6", 6, &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)]),
        // Two K3 triangles with a bridge
        (
            "two-k3-bridge",
            6,
            &[(0, 1), (1, 2), (0, 2), (3, 4), (4, 5), (3, 5), (2, 3)],
        ),
        // K5 — densely connected, encourages multiple local-move sweeps
        (
            "k5",
            5,
            &[
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 4),
                (1, 2),
                (1, 3),
                (1, 4),
                (2, 3),
                (2, 4),
                (3, 4),
            ],
        ),
    ];

    for &(name, n, edges) in graphs {
        let node_paths: Vec<String> = (0..n).map(|i| format!("n{i}.rs")).collect();
        let dg = build_dependency_graph_from_edges(&node_paths, edges);

        for seed in 0u64..=255 {
            let cfg = LeidenConfig {
                seed,
                ..LeidenConfig::default()
            };
            let p = run_leiden(&dg, &cfg, None);
            assert_eq!(
                p.assignments.len(),
                n,
                "graph={name} seed={seed}: partition covers {}/{n} nodes",
                p.assignments.len()
            );
        }
    }
}
