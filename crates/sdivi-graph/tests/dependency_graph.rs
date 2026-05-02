//! Tests for DependencyGraph accessor methods and import resolution strategies.
//!
//! Exercises: node_path, node_for_path, edges_as_pairs, neighbors, relative
//! path resolution, directory-module resolution, and ambiguous-stem handling.

use sdivi_graph::dependency_graph::build_dependency_graph;
use sdivi_parsing::feature_record::FeatureRecord;
use std::path::PathBuf;

fn make_record(path: &str, imports: &[&str]) -> FeatureRecord {
    FeatureRecord {
        path: PathBuf::from(path),
        language: "rust".to_string(),
        imports: imports.iter().map(|s| s.to_string()).collect(),
        exports: vec![],
        signatures: vec![],
        pattern_hints: vec![],
    }
}

// ── node_path / node_for_path ────────────────────────────────────────────────

#[test]
fn node_path_returns_path_for_valid_index() {
    let records = vec![
        make_record("src/lib.rs", &[]),
        make_record("src/models.rs", &[]),
    ];
    let dg = build_dependency_graph(&records);

    let idx0 = dg
        .node_for_path(&PathBuf::from("src/lib.rs"))
        .expect("lib.rs must be in graph");
    let idx1 = dg
        .node_for_path(&PathBuf::from("src/models.rs"))
        .expect("models.rs must be in graph");

    assert_eq!(
        dg.node_path(idx0),
        Some(PathBuf::from("src/lib.rs").as_path())
    );
    assert_eq!(
        dg.node_path(idx1),
        Some(PathBuf::from("src/models.rs").as_path())
    );
}

#[test]
fn node_path_returns_none_for_out_of_range_index() {
    let records = vec![make_record("src/lib.rs", &[])];
    let dg = build_dependency_graph(&records);
    assert!(
        dg.node_path(999).is_none(),
        "out-of-range index must return None"
    );
}

#[test]
fn node_for_path_returns_none_for_unknown_path() {
    let records = vec![make_record("src/lib.rs", &[])];
    let dg = build_dependency_graph(&records);
    let result = dg.node_for_path(&PathBuf::from("src/unknown.rs"));
    assert!(result.is_none(), "unknown path must return None");
}

#[test]
fn node_for_path_empty_graph_returns_none() {
    let dg = build_dependency_graph(&[]);
    assert!(dg.node_for_path(&PathBuf::from("src/lib.rs")).is_none());
}

// ── edges_as_pairs ───────────────────────────────────────────────────────────

#[test]
fn edges_as_pairs_empty_graph_returns_empty_vec() {
    let dg = build_dependency_graph(&[]);
    assert!(dg.edges_as_pairs().is_empty());
}

#[test]
fn edges_as_pairs_single_edge_correct_direction() {
    // lib.rs imports models.rs via `crate::models`
    let records = vec![
        make_record("src/lib.rs", &["crate::models"]),
        make_record("src/models.rs", &[]),
    ];
    let dg = build_dependency_graph(&records);

    let pairs = dg.edges_as_pairs();
    assert_eq!(pairs.len(), 1, "exactly one edge");

    let lib_idx = dg.node_for_path(&PathBuf::from("src/lib.rs")).unwrap();
    let models_idx = dg.node_for_path(&PathBuf::from("src/models.rs")).unwrap();

    assert!(
        pairs.contains(&(lib_idx, models_idx)),
        "edge must go from lib.rs ({lib_idx}) to models.rs ({models_idx}), got {pairs:?}"
    );
}

// ── neighbors ────────────────────────────────────────────────────────────────

#[test]
fn neighbors_returns_correct_targets() {
    // a.rs → b.rs and a.rs → c.rs
    let records = vec![
        make_record("src/a.rs", &["crate::b", "crate::c"]),
        make_record("src/b.rs", &[]),
        make_record("src/c.rs", &[]),
    ];
    let dg = build_dependency_graph(&records);

    let a_idx = dg.node_for_path(&PathBuf::from("src/a.rs")).unwrap();
    let b_idx = dg.node_for_path(&PathBuf::from("src/b.rs")).unwrap();
    let c_idx = dg.node_for_path(&PathBuf::from("src/c.rs")).unwrap();

    let mut nbrs = dg.neighbors(a_idx);
    nbrs.sort_unstable();

    assert!(nbrs.contains(&b_idx), "a.rs must list b.rs as neighbour");
    assert!(nbrs.contains(&c_idx), "a.rs must list c.rs as neighbour");
}

#[test]
fn neighbors_leaf_node_returns_empty() {
    let records = vec![
        make_record("src/a.rs", &["crate::b"]),
        make_record("src/b.rs", &[]),
    ];
    let dg = build_dependency_graph(&records);
    let b_idx = dg.node_for_path(&PathBuf::from("src/b.rs")).unwrap();
    assert!(dg.neighbors(b_idx).is_empty(), "b.rs has no outgoing edges");
}

// ── relative path import resolution ─────────────────────────────────────────

#[test]
fn relative_import_dot_slash_resolves_to_sibling() {
    // Python-style: `./utils` from `src/main.py` → `src/utils.py`
    let records = vec![
        make_record("src/main.py", &["./utils"]),
        make_record("src/utils.py", &[]),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        "relative ./utils must resolve to src/utils.py"
    );
}

#[test]
fn relative_import_parent_slash_strips_prefix_resolves_in_same_dir() {
    // The implementation strips the `../` prefix but then resolves in the
    // importer's own directory, not the parent.  `../shared` from
    // `src/sub/module.py` looks for `src/sub/shared.py` (same directory),
    // NOT `src/shared.py` (parent directory).
    // See BUG note in TESTER_REPORT.md re: parent navigation not implemented.
    let records = vec![
        make_record("src/sub/module.py", &["../shared"]),
        make_record("src/sub/shared.py", &[]), // same dir, not parent
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        "../shared strips prefix and resolves in same directory"
    );
}

#[test]
fn relative_import_unresolvable_drops_silently() {
    // `./nonexistent` has no matching file — must produce 0 edges, not a panic.
    let records = vec![make_record("src/main.py", &["./nonexistent"])];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        0,
        "unresolvable relative import must be dropped"
    );
}

// ── directory module resolution ───────────────────────────────────────────────

#[test]
fn relative_import_resolves_to_directory_index_ts() {
    // TypeScript: `./components` → `./components/index.ts`
    let records = vec![
        make_record("src/app.ts", &["./components"]),
        make_record("src/components/index.ts", &[]),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(dg.edge_count(), 1, "./components must resolve via index.ts");
}

#[test]
fn relative_import_resolves_to_mod_rs() {
    // Rust: `./parser` → `./parser/mod.rs`
    let records = vec![
        make_record("src/lib.rs", &["./parser"]),
        make_record("src/parser/mod.rs", &[]),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(dg.edge_count(), 1, "./parser must resolve via mod.rs");
}

// ── Rust crate:: prefix resolution ──────────────────────────────────────────

#[test]
fn crate_prefix_resolves_first_path_segment_only() {
    // `crate::models::User` should resolve to models.rs, ignoring `::User`
    let records = vec![
        make_record("src/lib.rs", &["crate::models::User"]),
        make_record("src/models.rs", &[]),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        "crate::models::User must resolve to models.rs"
    );
}

#[test]
fn self_prefix_resolves_to_sibling_stem() {
    let records = vec![
        make_record("src/lib.rs", &["self::utils"]),
        make_record("src/utils.rs", &[]),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(dg.edge_count(), 1, "self::utils must resolve to utils.rs");
}

#[test]
fn super_prefix_resolves_to_stem() {
    let records = vec![
        make_record("src/sub/mod.rs", &["super::config"]),
        make_record("src/config.rs", &[]),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        "super::config must resolve to config.rs"
    );
}

// ── ambiguous stem (two files share the same stem) ──────────────────────────

#[test]
fn ambiguous_stem_does_not_add_edge() {
    // Both `src/models.rs` and `lib/models.rs` share stem "models".
    // Resolution is ambiguous — no edge should be added.
    let records = vec![
        make_record("src/main.rs", &["crate::models"]),
        make_record("src/models.rs", &[]),
        make_record("lib/models.rs", &[]),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        0,
        "ambiguous stem must not produce an edge"
    );
}

// ── external imports are dropped ─────────────────────────────────────────────

#[test]
fn external_crate_import_produces_no_edge() {
    let records = vec![
        make_record("src/lib.rs", &["serde::Serialize", "tokio::runtime"]),
        make_record("src/models.rs", &[]),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(dg.edge_count(), 0, "external imports must produce no edges");
}
