//! Property-based tests for `sdivi_core::compute_boundary_violations`.

use proptest::prelude::*;
use sdivi_core::{
    compute_boundary_violations, BoundaryDefInput, BoundarySpecInput, DependencyGraphInput,
    EdgeInput, NodeInput,
};

fn node(id: &str) -> NodeInput {
    NodeInput { id: id.into(), path: id.into(), language: "rust".into() }
}

fn edge(src: &str, tgt: &str) -> EdgeInput {
    EdgeInput { source: src.into(), target: tgt.into() }
}

fn boundary(name: &str, modules: &[&str], allow: &[&str]) -> BoundaryDefInput {
    BoundaryDefInput {
        name: name.into(),
        modules: modules.iter().map(|s| s.to_string()).collect(),
        allow_imports_from: allow.iter().map(|s| s.to_string()).collect(),
    }
}

proptest! {
    #[test]
    fn prop_violation_count_equals_violations_len(
        n_nodes in 0usize..10,
        n_edges in 0usize..20,
    ) {
        let spec = BoundarySpecInput {
            boundaries: vec![
                boundary("api", &["crates/api/**"], &["db"]),
                boundary("db", &["crates/db/**"], &[]),
            ],
        };
        let nodes: Vec<NodeInput> = (0..n_nodes)
            .map(|i| node(&format!("crates/a/f{i}.rs")))
            .collect();
        let edges: Vec<EdgeInput> = (0..n_edges)
            .map(|i| {
                let src = i % n_nodes.max(1);
                let tgt = (i + 1) % n_nodes.max(1);
                edge(&format!("crates/a/f{src}.rs"), &format!("crates/a/f{tgt}.rs"))
            })
            .collect();
        let g = DependencyGraphInput { nodes, edges };
        let r = compute_boundary_violations(&g, &spec).unwrap();
        prop_assert_eq!(r.violation_count as usize, r.violations.len());
    }

    #[test]
    fn prop_violations_are_sorted(
        n_nodes in 2usize..8,
        n_edges in 0usize..15,
    ) {
        let nodes: Vec<NodeInput> = (0..n_nodes).map(|i| {
            let bnd = if i % 2 == 0 { "a" } else { "b" };
            node(&format!("crates/{bnd}/f{i}.rs"))
        }).collect();
        let edges: Vec<EdgeInput> = (0..n_edges).map(|i| {
            let src = i % n_nodes;
            let tgt = (i * 3 + 1) % n_nodes;
            let bnd_s = if src % 2 == 0 { "a" } else { "b" };
            let bnd_t = if tgt % 2 == 0 { "a" } else { "b" };
            edge(
                &format!("crates/{bnd_s}/f{src}.rs"),
                &format!("crates/{bnd_t}/f{tgt}.rs"),
            )
        }).collect();
        let g = DependencyGraphInput { nodes, edges };
        let spec = BoundarySpecInput {
            boundaries: vec![
                boundary("a", &["crates/a/**"], &[]),
                boundary("b", &["crates/b/**"], &[]),
            ],
        };
        let r = compute_boundary_violations(&g, &spec).unwrap();
        let mut sorted = r.violations.clone();
        sorted.sort();
        prop_assert_eq!(r.violations, sorted, "violations must be lexicographically sorted");
    }
}
