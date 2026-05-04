//! Unit tests for `sdivi_core::compute_boundary_violations`.

use sdivi_core::{
    compute_boundary_violations, BoundaryDefInput, BoundarySpecInput, DependencyGraphInput,
    EdgeInput, NodeInput,
};

// ── helpers ──────────────────────────────────────────────────────────────────

fn node(id: &str) -> NodeInput {
    NodeInput {
        id: id.into(),
        path: id.into(),
        language: "rust".into(),
    }
}

fn edge(src: &str, tgt: &str) -> EdgeInput {
    EdgeInput {
        source: src.into(),
        target: tgt.into(),
    }
}

fn boundary(name: &str, modules: &[&str], allow: &[&str]) -> BoundaryDefInput {
    BoundaryDefInput {
        name: name.into(),
        modules: modules.iter().map(|s| s.to_string()).collect(),
        allow_imports_from: allow.iter().map(|s| s.to_string()).collect(),
    }
}

fn two_boundary_spec() -> BoundarySpecInput {
    BoundarySpecInput {
        boundaries: vec![
            boundary("api", &["crates/api/**"], &["db"]),
            boundary("db", &["crates/db/**"], &[]),
        ],
    }
}

// ── empty spec / empty graph ──────────────────────────────────────────────────

#[test]
fn empty_spec_returns_zero() {
    let g = DependencyGraphInput {
        nodes: vec![],
        edges: vec![],
    };
    let r = compute_boundary_violations(&g, &BoundarySpecInput { boundaries: vec![] }).unwrap();
    assert_eq!(r.violation_count, 0);
    assert!(r.violations.is_empty());
}

#[test]
fn empty_graph_with_spec_returns_zero() {
    let g = DependencyGraphInput {
        nodes: vec![],
        edges: vec![],
    };
    let r = compute_boundary_violations(&g, &two_boundary_spec()).unwrap();
    assert_eq!(r.violation_count, 0);
}

// ── acceptance criterion: db → api = 1 violation ─────────────────────────────

#[test]
fn db_to_api_is_violation() {
    let g = DependencyGraphInput {
        nodes: vec![node("crates/db/foo.rs"), node("crates/api/bar.rs")],
        edges: vec![edge("crates/db/foo.rs", "crates/api/bar.rs")],
    };
    let r = compute_boundary_violations(&g, &two_boundary_spec()).unwrap();
    assert_eq!(r.violation_count, 1);
    assert_eq!(
        r.violations[0],
        ("crates/db/foo.rs".into(), "crates/api/bar.rs".into())
    );
}

#[test]
fn api_to_db_is_allowed() {
    let g = DependencyGraphInput {
        nodes: vec![node("crates/api/bar.rs"), node("crates/db/foo.rs")],
        edges: vec![edge("crates/api/bar.rs", "crates/db/foo.rs")],
    };
    let r = compute_boundary_violations(&g, &two_boundary_spec()).unwrap();
    assert_eq!(r.violation_count, 0);
}

// ── unscoped nodes ────────────────────────────────────────────────────────────

#[test]
fn unscoped_source_no_violation() {
    let g = DependencyGraphInput {
        nodes: vec![node("src/main.rs"), node("crates/api/bar.rs")],
        edges: vec![edge("src/main.rs", "crates/api/bar.rs")],
    };
    let r = compute_boundary_violations(&g, &two_boundary_spec()).unwrap();
    assert_eq!(r.violation_count, 0);
}

#[test]
fn unscoped_target_no_violation() {
    let g = DependencyGraphInput {
        nodes: vec![node("crates/db/foo.rs"), node("src/utils.rs")],
        edges: vec![edge("crates/db/foo.rs", "src/utils.rs")],
    };
    let r = compute_boundary_violations(&g, &two_boundary_spec()).unwrap();
    assert_eq!(r.violation_count, 0);
}

// ── same-boundary edges ───────────────────────────────────────────────────────

#[test]
fn same_boundary_edge_is_not_violation() {
    let g = DependencyGraphInput {
        nodes: vec![node("crates/api/a.rs"), node("crates/api/b.rs")],
        edges: vec![edge("crates/api/a.rs", "crates/api/b.rs")],
    };
    let r = compute_boundary_violations(&g, &two_boundary_spec()).unwrap();
    assert_eq!(r.violation_count, 0);
}

#[test]
fn self_loop_is_not_violation() {
    let g = DependencyGraphInput {
        nodes: vec![node("crates/db/foo.rs")],
        edges: vec![edge("crates/db/foo.rs", "crates/db/foo.rs")],
    };
    let r = compute_boundary_violations(&g, &two_boundary_spec()).unwrap();
    assert_eq!(r.violation_count, 0);
}

// ── allow_imports_from is not transitive ──────────────────────────────────────

#[test]
fn allow_is_not_transitive() {
    // a allows b, b allows c; a → c is still a violation.
    let spec = BoundarySpecInput {
        boundaries: vec![
            boundary("a", &["crates/a/**"], &["b"]),
            boundary("b", &["crates/b/**"], &["c"]),
            boundary("c", &["crates/c/**"], &[]),
        ],
    };
    let g = DependencyGraphInput {
        nodes: vec![node("crates/a/x.rs"), node("crates/c/z.rs")],
        edges: vec![edge("crates/a/x.rs", "crates/c/z.rs")],
    };
    let r = compute_boundary_violations(&g, &spec).unwrap();
    assert_eq!(r.violation_count, 1);
}

// ── match_boundary: most-specific glob wins ───────────────────────────────────

#[test]
fn specific_glob_wins_over_generic() {
    // Both `crates/**` and `crates/api/**` match `crates/api/foo.rs`.
    // `crates/api/**` is longer → api wins.
    let spec = BoundarySpecInput {
        boundaries: vec![
            boundary("generic", &["crates/**"], &[]),
            boundary("api", &["crates/api/**"], &[]),
        ],
    };
    // An edge from crates/api/foo.rs to a "generic" node (crates/other/x.rs).
    // If `api` wins for crates/api/foo.rs, and `generic` wins for crates/other/x.rs,
    // and `api.allow_imports_from` is empty → violation.
    let g = DependencyGraphInput {
        nodes: vec![node("crates/api/foo.rs"), node("crates/other/x.rs")],
        edges: vec![edge("crates/api/foo.rs", "crates/other/x.rs")],
    };
    let r = compute_boundary_violations(&g, &spec).unwrap();
    // generic node: matched by `crates/**` → boundary = generic.
    // api node: matched by both, longer wins → boundary = api.
    // api.allow_imports_from = [] → violation.
    assert_eq!(r.violation_count, 1);
}

// ── violations sorted lexicographically ──────────────────────────────────────

#[test]
fn violations_sorted_lexicographically() {
    let spec = BoundarySpecInput {
        boundaries: vec![boundary("a", &["a/**"], &[]), boundary("b", &["b/**"], &[])],
    };
    let g = DependencyGraphInput {
        nodes: vec![
            node("a/z.rs"),
            node("b/z.rs"),
            node("a/a.rs"),
            node("b/a.rs"),
        ],
        edges: vec![edge("a/z.rs", "b/z.rs"), edge("a/a.rs", "b/a.rs")],
    };
    let r = compute_boundary_violations(&g, &spec).unwrap();
    assert_eq!(r.violation_count, 2);
    assert!(
        r.violations[0] <= r.violations[1],
        "violations must be sorted"
    );
    let mut expected = r.violations.clone();
    expected.sort();
    assert_eq!(r.violations, expected);
}

// ── match_boundary tie-break by boundary name ─────────────────────────────────

#[test]
fn tie_break_by_boundary_name_ascending() {
    // Two boundaries with same-length glob patterns both matching the node.
    // "aaa" < "bbb" → aaa wins.
    let spec = BoundarySpecInput {
        boundaries: vec![
            boundary("bbb", &["src/x.rs"], &[]),
            boundary("aaa", &["src/x.rs"], &[]),
        ],
    };
    // src/x.rs → src/y.rs; src/y.rs unscoped → no violation regardless.
    // We verify the node is classified under "aaa" by checking no violation
    // when aaa.allow_imports_from includes the target boundary.
    let spec2 = BoundarySpecInput {
        boundaries: vec![
            boundary("bbb", &["src/y.rs"], &[]),
            boundary("aaa", &["src/x.rs"], &["bbb"]),
        ],
    };
    let g = DependencyGraphInput {
        nodes: vec![node("src/x.rs"), node("src/y.rs")],
        edges: vec![edge("src/x.rs", "src/y.rs")],
    };
    // Under "aaa" classification, src/x.rs → src/y.rs is allowed (aaa allows bbb).
    let r = compute_boundary_violations(&g, &spec2).unwrap();
    assert_eq!(
        r.violation_count, 0,
        "aaa should win tie and allow the import"
    );

    // Under "bbb" classification (wrong), it would be a violation.
    // This confirms "aaa" was selected.
    let spec3 = BoundarySpecInput {
        boundaries: vec![
            boundary("bbb", &["src/x.rs", "src/y.rs"], &[]),
            boundary("aaa", &["src/x.rs"], &[]),
        ],
    };
    // src/x.rs: "src/x.rs" (len 9) matches both, tie → "aaa" wins.
    // src/y.rs: only "src/x.rs" in bbb...wait, bbb has ["src/x.rs","src/y.rs"].
    // src/y.rs matches bbb's "src/y.rs" (len 9) but not aaa.
    // So x→y: aaa→bbb. aaa.allow_imports_from=[] → violation.
    let r2 = compute_boundary_violations(&g, &spec3).unwrap();
    assert_eq!(
        r2.violation_count, 1,
        "aaa wins and has no allow list → violation"
    );
    let _ = spec;
}

// ── duplicate-edge behavior ───────────────────────────────────────────────────

/// Verifies that when the same (source, target) edge pair appears twice in
/// `graph.edges`, the implementation counts each occurrence independently —
/// producing two violation entries (and violation_count = 2) rather than
/// deduplicating to one.
///
/// This documents current behavior: `compute_boundary_violations` is a pure
/// function over its input; callers (e.g. sdivi-pipeline's graph builder) are
/// responsible for ensuring no duplicate edges reach it.
#[test]
fn duplicate_edge_produces_two_violations() {
    let g = DependencyGraphInput {
        nodes: vec![node("crates/db/foo.rs"), node("crates/api/bar.rs")],
        edges: vec![
            edge("crates/db/foo.rs", "crates/api/bar.rs"),
            edge("crates/db/foo.rs", "crates/api/bar.rs"),
        ],
    };
    let r = compute_boundary_violations(&g, &two_boundary_spec()).unwrap();
    assert_eq!(
        r.violation_count, 2,
        "each duplicate edge is counted as a separate violation"
    );
    assert_eq!(r.violations.len(), 2);
    // Both entries are the same pair (sorted output is still two entries).
    assert_eq!(r.violations[0], r.violations[1]);
}

/// Verifies that a duplicate edge within the same boundary does NOT produce
/// violations — each occurrence is still a same-boundary edge and is skipped.
#[test]
fn duplicate_same_boundary_edge_produces_no_violation() {
    let g = DependencyGraphInput {
        nodes: vec![node("crates/api/a.rs"), node("crates/api/b.rs")],
        edges: vec![
            edge("crates/api/a.rs", "crates/api/b.rs"),
            edge("crates/api/a.rs", "crates/api/b.rs"),
        ],
    };
    let r = compute_boundary_violations(&g, &two_boundary_spec()).unwrap();
    assert_eq!(r.violation_count, 0);
    assert!(r.violations.is_empty());
}
