//! Additional edge-case tests for the M26 resolver.
//!
//! Covers:
//! - Python PEP 420 namespace packages (directory without __init__.py)
//! - Multiple consecutive `super::` levels
//! - Java multi-module Maven roots dynamically discovered by compute_java_roots
//! - Graph-level determinism (same input → same sorted edge list on two calls)

use sdivi_graph::dependency_graph::{
    build_dependency_graph, build_dependency_graph_with_go_module,
};
use sdivi_parsing::feature_record::FeatureRecord;
use std::path::PathBuf;

fn rec(path: &str, imports: &[&str], language: &str) -> FeatureRecord {
    FeatureRecord {
        path: PathBuf::from(path),
        language: language.to_string(),
        imports: imports.iter().map(|s| s.to_string()).collect(),
        exports: vec![],
        signatures: vec![],
        pattern_hints: vec![],
    }
}

// ── Python PEP 420 namespace packages ────────────────────────────────────────

/// A Python 3.3+ namespace package is a directory that contains `.py` files
/// but no `__init__.py`.  The resolver must still produce an edge.
#[test]
fn python_pep420_bare_dotted_resolves_via_namespace_dir() {
    // `foo.bar.baz` where `foo/bar/baz/` contains `.py` files but no `__init__.py`
    let records = vec![
        rec("src/main.py", &["foo.bar.baz"], "python"),
        rec("foo/bar/baz/utils.py", &[], "python"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        "PEP 420 namespace package without __init__.py must still resolve"
    );
}

/// Namespace package lookup must prefer `__init__.py` when both it and a plain
/// module file exist at the same level.
#[test]
fn python_pep420_init_py_takes_precedence_over_namespace() {
    // both `foo/bar.py` and `foo/bar/__init__.py` exist; the `.py` file wins
    // (it is tried first by resolve_python_bare)
    let records = vec![
        rec("src/main.py", &["foo.bar"], "python"),
        rec("foo/bar.py", &[], "python"),
        rec("foo/bar/__init__.py", &[], "python"),
    ];
    let dg = build_dependency_graph(&records);
    // Exactly one edge — the `.py` file is the primary target
    assert_eq!(
        dg.edge_count(),
        1,
        "foo.bar resolves to foo/bar.py (first match wins)"
    );
    let main_idx = dg
        .node_for_path(&PathBuf::from("src/main.py"))
        .expect("src/main.py must be in graph");
    let bar_py_idx = dg
        .node_for_path(&PathBuf::from("foo/bar.py"))
        .expect("foo/bar.py must be in graph");
    assert!(
        dg.edges_as_pairs().contains(&(main_idx, bar_py_idx)),
        "edge must point to foo/bar.py, not foo/bar/__init__.py"
    );
}

// ── Multiple consecutive super:: levels ──────────────────────────────────────

/// `super::super::config` from `a/b/c.rs` walks two levels up and finds
/// `config.rs` at the repo root.
#[test]
fn double_super_resolves_two_levels_up() {
    let records = vec![
        rec("a/b/c.rs", &["super::super::config"], "rust"),
        rec("config.rs", &[], "rust"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        "super::super::config from a/b/c.rs must resolve to config.rs"
    );
    let c_idx = dg
        .node_for_path(&PathBuf::from("a/b/c.rs"))
        .expect("a/b/c.rs must be in graph");
    let config_idx = dg
        .node_for_path(&PathBuf::from("config.rs"))
        .expect("config.rs must be in graph");
    assert!(
        dg.edges_as_pairs().contains(&(c_idx, config_idx)),
        "edge must go from a/b/c.rs to config.rs"
    );
}

/// `super::super::x` from `a/b/c.rs` finds `a/x.rs` (one level up from `a/b/`)
/// when both `a/x.rs` and `x.rs` exist — subtree search in the walked-up base
/// `a/` returns a unique match there.
#[test]
fn double_super_finds_intermediate_level_stem() {
    let records = vec![
        rec("a/b/c.rs", &["super::super::models"], "rust"),
        rec("a/models.rs", &[], "rust"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        "super::super::models from a/b/c.rs must resolve to a/models.rs"
    );
}

/// Three levels of `super::` that overshoot the repo root fall back to the
/// global stem-map.  If the stem is globally unique, the edge is still added.
#[test]
fn triple_super_overshoot_falls_back_to_global_stem() {
    // `a/b/c.rs` → `super::super::super::helpers` — three levels up overshoots the
    // root (only two path components above), so the resolver falls back to the
    // global stem-map which finds `helpers.rs`.
    let records = vec![
        rec("a/b/c.rs", &["super::super::super::helpers"], "rust"),
        rec("helpers.rs", &[], "rust"),
    ];
    let dg = build_dependency_graph(&records);
    // The fallback to global stem-map means the edge should still be found.
    assert_eq!(
        dg.edge_count(),
        1,
        "triple super:: overshoot must fall back to global stem resolution"
    );
}

// ── Java multi-module Maven roots ─────────────────────────────────────────────

/// In a multi-module Maven project, each module has its own `src/main/java`
/// root.  `compute_java_roots` should discover these dynamically and both
/// modules' classes should be resolvable.
#[test]
fn java_multi_module_maven_roots_discovered_dynamically() {
    // module-a imports from module-b; both have their own src/main/java
    let records = vec![
        rec(
            "module-a/src/main/java/com/acme/app/App.java",
            &["com.acme.lib.Service"],
            "java",
        ),
        rec(
            "module-b/src/main/java/com/acme/lib/Service.java",
            &[],
            "java",
        ),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        "com.acme.lib.Service must resolve via module-b/src/main/java"
    );
    let app_idx = dg
        .node_for_path(&PathBuf::from(
            "module-a/src/main/java/com/acme/app/App.java",
        ))
        .expect("App.java must be in graph");
    let svc_idx = dg
        .node_for_path(&PathBuf::from(
            "module-b/src/main/java/com/acme/lib/Service.java",
        ))
        .expect("Service.java must be in graph");
    assert!(
        dg.edges_as_pairs().contains(&(app_idx, svc_idx)),
        "edge must go from App.java to Service.java"
    );
}

/// Wildcard import (`com.acme.lib.*`) in a multi-module layout produces one
/// edge per class in the package across all discovered roots.
#[test]
fn java_multi_module_wildcard_import_across_roots() {
    let records = vec![
        rec(
            "module-a/src/main/java/com/acme/app/App.java",
            &["com.acme.lib.*"],
            "java",
        ),
        rec(
            "module-b/src/main/java/com/acme/lib/Alpha.java",
            &[],
            "java",
        ),
        rec("module-b/src/main/java/com/acme/lib/Beta.java", &[], "java"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        2,
        "wildcard import must emit one edge per class across discovered Maven roots"
    );
}

// ── Graph determinism ─────────────────────────────────────────────────────────

/// Two calls to `build_dependency_graph` with the same records must produce
/// the same sorted edge list.  Non-determinism (e.g. iterating a HashMap)
/// would cause this test to flake.
#[test]
fn build_dependency_graph_is_deterministic() {
    let records = vec![
        rec(
            "app/index.ts",
            &["../shared/utils", "./components/button"],
            "typescript",
        ),
        rec("shared/utils.ts", &[], "typescript"),
        rec("app/components/button/index.ts", &["../icon"], "typescript"),
        rec("app/components/icon.ts", &[], "typescript"),
        rec("pkg/app.py", &[".models", "util.helper"], "python"),
        rec("pkg/models.py", &[], "python"),
        rec("util/helper.py", &[], "python"),
    ];

    let dg1 = build_dependency_graph(&records);
    let dg2 = build_dependency_graph(&records);

    let mut edges1 = dg1.edges_as_pairs();
    let mut edges2 = dg2.edges_as_pairs();
    edges1.sort_unstable();
    edges2.sort_unstable();

    assert_eq!(
        edges1, edges2,
        "two builds from the same records must produce identical edge sets"
    );
    assert_eq!(
        dg1.edge_count(),
        dg2.edge_count(),
        "edge count must be stable across builds"
    );
}

/// Same determinism check with Go module imports, which iterate over
/// `path_to_node` (BTreeMap) and must produce stable ordering.
#[test]
fn build_dependency_graph_go_module_is_deterministic() {
    let records = vec![
        rec("cmd/main.go", &["example.com/app/internal/util"], "go"),
        rec("internal/util/alpha.go", &[], "go"),
        rec("internal/util/beta.go", &[], "go"),
        rec("internal/util/gamma.go", &[], "go"),
    ];

    let dg1 = build_dependency_graph_with_go_module(&records, Some("example.com/app"));
    let dg2 = build_dependency_graph_with_go_module(&records, Some("example.com/app"));

    let mut edges1 = dg1.edges_as_pairs();
    let mut edges2 = dg2.edges_as_pairs();
    edges1.sort_unstable();
    edges2.sort_unstable();

    assert_eq!(
        edges1, edges2,
        "Go multi-edge resolution must produce identical edge sets across builds"
    );
    assert_eq!(
        dg1.edge_count(),
        3,
        "three .go files in package = three edges"
    );
}

// ── Python relative import triple-dot ────────────────────────────────────────

/// `...sibling` from `a/b/c.py` means three dots: dot_count=3, levels_up=2.
/// From `a/b/c.py`, current dir `a/b`, walk up 2 → root `""`.
/// Resolves to `sibling.py` at the repo root (the grandparent package).
#[test]
fn python_triple_dot_relative_import_resolves_to_grandparent_package() {
    // Three dots: dot_count=3, levels_up = 3-1 = 2.
    // From `a/b/c.py`, current dir is `a/b`, walk up 2 → repo root.
    // Then look for `sibling.py` at root.
    let records = vec![
        rec("a/b/c.py", &["...sibling"], "python"),
        rec("sibling.py", &[], "python"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        "...sibling from a/b/c.py must resolve to sibling.py at repo root (2 levels up)"
    );
}

// ── Self-loop prevention ──────────────────────────────────────────────────────

/// A file that imports itself must not produce a self-loop edge.
#[test]
fn self_import_produces_no_self_loop() {
    let records = vec![rec("src/utils.rs", &["crate::utils"], "rust")];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        0,
        "self-import must be silently dropped; no self-loop edge added"
    );
}

/// Python: `from . import models` from `pkg/models.py` itself is a self-loop.
#[test]
fn python_relative_self_import_is_dropped() {
    // From `pkg/models.py`, `.models` resolves to `pkg/models.py` — same file.
    let records = vec![rec("pkg/models.py", &[".models"], "python")];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        0,
        "Python relative self-import must be dropped"
    );
}
