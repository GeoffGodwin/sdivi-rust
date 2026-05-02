//! Unit, integration, and property tests for the Leiden refinement phase.

use proptest::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng as _;
use sdivi_detection::internal::{
    compute_modularity, refine_partition, well_connected, LeidenGraph, RefinementState,
};
use sdivi_detection::leiden::run_leiden;
use sdivi_detection::partition::{LeidenConfig, QualityFunction};
use sdivi_graph::dependency_graph::build_dependency_graph_from_edges;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

// ── Unit: RefinementState::from_partition ────────────────────────────────────

#[test]
fn from_partition_singleton_init_sigma_tot() {
    // 3-node path: degrees = [1, 2, 1], total_weight = 2.
    let g = LeidenGraph::from_edges(3, &[(0, 1), (1, 2)]);
    let partition: Vec<usize> = (0..3).collect();
    let state = RefinementState::from_partition(&g, &partition, 3);

    assert_eq!(state.assignment, vec![0, 1, 2]);
    assert_eq!(state.size, vec![1, 1, 1]);
    assert!((state.sigma_tot[0] - g.degree[0]).abs() < 1e-12);
    assert!((state.sigma_tot[1] - g.degree[1]).abs() < 1e-12);
    assert!((state.sigma_tot[2] - g.degree[2]).abs() < 1e-12);
    // Singleton inner_edges = self-loops only (none here).
    assert!((state.inner_edges[0]).abs() < 1e-12);
    assert!((state.inner_edges[1]).abs() < 1e-12);
    assert!((state.inner_edges[2]).abs() < 1e-12);
}

#[test]
fn from_partition_non_singleton_inner_edges() {
    // Triangle: nodes 0,1,2 all in community 0.
    let g = LeidenGraph::from_edges(3, &[(0, 1), (1, 2), (0, 2)]);
    let partition = vec![0usize, 0, 0];
    let state = RefinementState::from_partition(&g, &partition, 1);

    assert_eq!(state.size[0], 3);
    assert!((state.sigma_tot[0] - 6.0).abs() < 1e-12); // 3 nodes × degree 2
                                                       // inner_edges for all-in-one = 3 edges total.
    assert!((state.inner_edges[0] - 3.0).abs() < 1e-12);
}

// ── Unit: RefinementState::apply_move ────────────────────────────────────────

#[test]
fn apply_move_updates_sigma_tot_and_size() {
    // Path 0-1-2.  Singleton init.  Move node 0 from comm 0 to comm 1.
    let g = LeidenGraph::from_edges(3, &[(0, 1), (1, 2)]);
    let partition: Vec<usize> = (0..3).collect();
    let mut state = RefinementState::from_partition(&g, &partition, 3);

    state.apply_move(&g, 0, 0, 1);

    assert_eq!(state.assignment[0], 1);
    assert_eq!(state.size[0], 0);
    assert_eq!(state.size[1], 2);
    assert!((state.sigma_tot[0]).abs() < 1e-12);
    assert!((state.sigma_tot[1] - (g.degree[0] + g.degree[1])).abs() < 1e-12);
    // Edge 0→1 is now internal to comm 1.
    assert!((state.inner_edges[1] - 1.0).abs() < 1e-12);
}

// ── Unit: well_connected ─────────────────────────────────────────────────────

#[test]
fn well_connected_gamma_zero_always_true() {
    assert!(well_connected(0.0, 1, 10, 0.0));
    assert!(well_connected(0.0, 5, 10, 0.0));
}

#[test]
fn well_connected_strong_connection_passes() {
    // size_s=10, size_candidate=3: threshold = 1*(3 - 9/10) ≈ 2.1
    assert!(well_connected(3.0, 3, 10, 1.0));
    // Compute the threshold the same way the implementation does and add 1e-9 margin
    // to avoid relying on exact floating-point equality with the 2.1 literal, which
    // is not finitely representable in binary IEEE 754.
    let threshold = 1.0_f64 * (3.0_f64 - 9.0_f64 / 10.0_f64);
    assert!(well_connected(threshold + 1e-9, 3, 10, 1.0));
}

#[test]
fn well_connected_size_s_zero_always_true() {
    // size_s == 0 is an early-return short-circuit; must pass regardless of other args.
    assert!(well_connected(0.0, 0, 0, 1.0));
    assert!(well_connected(0.0, 5, 0, 1.0));
    assert!(well_connected(0.0, 3, 0, 10.0));
}

#[test]
fn well_connected_weak_connection_fails() {
    // size_s=10, size_candidate=3: threshold = 2.1
    assert!(!well_connected(2.0, 3, 10, 1.0));
    // size_s=6, size_candidate=1: threshold = 1*(1 - 1/6) = 0.833
    assert!(!well_connected(0.5, 1, 6, 1.0));
}

// ── Integration: two disconnected groups ─────────────────────────────────────

/// Two K3 triangles share one coarse community but have NO cross-edges.
/// Refinement must never put a node from {0,1,2} in the same sub-community
/// as a node from {3,4,5}: the `in_coarse` filter makes cross-group merges
/// impossible.
#[test]
fn two_disconnected_groups_never_mix_after_refine() {
    let g = LeidenGraph::from_edges(6, &[(0, 1), (1, 2), (0, 2), (3, 4), (4, 5), (3, 5)]);
    let coarse = vec![0usize; 6]; // all in one coarse community

    let refined = refine_partition(
        &g,
        &coarse,
        &mut make_rng(42),
        &QualityFunction::Modularity,
        1.0,
    );

    // Groups must be in disjoint sub-communities.
    let group_a = [refined[0], refined[1], refined[2]];
    let group_b = [refined[3], refined[4], refined[5]];
    for &ca in &group_a {
        for &cb in &group_b {
            assert_ne!(
                ca, cb,
                "node from {{0,1,2}} must not share a sub-community with {{3,4,5}}"
            );
        }
    }
}

/// Coarse boundary invariant: when the correct 2-community partition is
/// supplied, refinement must not mix nodes across coarse communities.
#[test]
fn refine_preserves_coarse_community_boundary() {
    let g = LeidenGraph::from_edges(6, &[(0, 1), (1, 2), (0, 2), (3, 4), (4, 5), (3, 5), (2, 3)]);
    let coarse = vec![0, 0, 0, 1, 1, 1];

    let refined = refine_partition(
        &g,
        &coarse,
        &mut make_rng(42),
        &QualityFunction::Modularity,
        1.0,
    );

    // No node from coarse-comm-0 may share a sub-community with a node from coarse-comm-1.
    let comm0: Vec<usize> = (0..3).map(|i| refined[i]).collect();
    let comm1: Vec<usize> = (3..6).map(|i| refined[i]).collect();
    for &c0 in &comm0 {
        for &c1 in &comm1 {
            assert_ne!(c0, c1, "coarse boundary must be respected");
        }
    }
}

/// Path graph: refinement respects the coarse community boundary regardless
/// of how many sub-communities are produced within it.
#[test]
fn refine_path_graph_boundary() {
    let g = LeidenGraph::from_edges(6, &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)]);
    let coarse = vec![0, 0, 0, 1, 1, 1];

    let refined = refine_partition(
        &g,
        &coarse,
        &mut make_rng(42),
        &QualityFunction::Modularity,
        1.0,
    );

    let set0: std::collections::BTreeSet<usize> = (0..3).map(|i| refined[i]).collect();
    let set1: std::collections::BTreeSet<usize> = (3..6).map(|i| refined[i]).collect();
    for &c0 in &set0 {
        assert!(!set1.contains(&c0), "coarse boundary must be respected");
    }
}

// ── Integration: full Leiden quality ─────────────────────────────────────────

/// Full Leiden on a ring-of-3-cliques must produce positive modularity,
/// confirming that the corrected refinement no longer collapses all nodes into
/// one community.
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
        "Leiden with corrected refinement must produce positive modularity (got {})",
        p.modularity
    );
    assert!(
        p.community_count() >= 2,
        "Two-clique graph must produce ≥ 2 communities (got {})",
        p.community_count()
    );
}

// ── Property: coarse-community subset invariant ───────────────────────────────
fn check_coarse_invariant(n: usize, raw_edges: &[(usize, usize)], coarse_raw: &[usize]) {
    let coarse: Vec<usize> = coarse_raw[..n].iter().map(|&c| c % 3).collect();
    let g = LeidenGraph::from_edges(n, raw_edges);
    let mut rng = make_rng(42);
    let refined = refine_partition(&g, &coarse, &mut rng, &QualityFunction::Modularity, 1.0);

    // For every pair in the same refined community, they must be in the same
    // coarse community.  This is the key correctness invariant of Leiden
    // refinement: sub-communities are always subsets of coarse communities.
    for i in 0..n {
        for j in 0..n {
            if refined[i] == refined[j] {
                assert_eq!(
                    coarse[i], coarse[j],
                    "nodes {i} and {j} share refined comm but different coarse comms"
                );
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn prop_refine_does_not_increase_coarse_communities(
        n in 3usize..=8usize,
        raw_edges in prop::collection::vec(
            (0usize..8usize, 0usize..8usize),
            0..16,
        ),
        coarse_raw in prop::collection::vec(0usize..=2usize, 8),
    ) {
        let edges: Vec<(usize, usize)> = raw_edges.into_iter()
            .map(|(u, v)| (u % n, v % n))
            .filter(|(u, v)| u != v) // no self-loops in the base graph
            .collect();
        check_coarse_invariant(n, &edges, &coarse_raw);
    }

    /// Leiden must produce modularity >= the all-singletons baseline (≤ 0).
    /// (Leiden monotone-improvement guarantee: only positive-gain moves are made.)
    #[test]
    fn prop_refine_modularity_does_not_decrease(
        n in 3usize..=8usize,
        raw_edges in prop::collection::vec((0usize..8usize, 0usize..8usize), 1..16),
        seed in 0u64..=255u64,
    ) {
        let edges: Vec<(usize, usize)> = raw_edges
            .into_iter()
            .map(|(u, v)| (u % n, v % n))
            .filter(|(u, v)| u != v)
            .collect();
        if edges.is_empty() {
            return Ok(());
        }
        let node_paths: Vec<String> = (0..n).map(|i| format!("n{i}.rs")).collect();
        let dg = build_dependency_graph_from_edges(&node_paths, &edges);
        let g = LeidenGraph::from_edges(n, &edges);
        if g.total_weight < 1e-12 {
            return Ok(());
        }
        let q_baseline = compute_modularity(&g, &(0..n).collect::<Vec<_>>());
        let cfg = LeidenConfig { seed, ..LeidenConfig::default() };
        let p = run_leiden(&dg, &cfg, None);
        let asgn: Vec<usize> = (0..n).map(|i| p.community_of(i).unwrap_or(0)).collect();
        let q_leiden = compute_modularity(&g, &asgn);
        prop_assert!(q_leiden >= q_baseline - 1e-9, "Q={q_leiden:.6} < baseline {q_baseline:.6}");
    }
}
