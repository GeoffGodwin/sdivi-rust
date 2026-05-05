//! Unit tests for M26 resolver improvements: parent navigation, per-language
//! dispatch, Python dotted/relative, Go module, and Java dotted/wildcard.

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

// ── relative path: multi-level parent ────────────────────────────────────────

#[test]
fn multi_level_parent_resolves_correctly() {
    // `../../shared/x` from `app/items/Edit.tsx` → `shared/x.tsx`
    let records = vec![
        rec("app/items/Edit.tsx", &["../../shared/x"], "typescript"),
        rec("shared/x.tsx", &[], "typescript"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        "../../shared/x must resolve to shared/x.tsx"
    );
}

#[test]
fn overshoot_relative_import_is_dropped() {
    // More `../` than path depth — must not panic, must produce zero edges.
    let records = vec![
        rec("src/util.ts", &["../../../../foo"], "typescript"),
        rec("foo.ts", &[], "typescript"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(dg.edge_count(), 0, "overshoot must not resolve to any node");
}

#[test]
fn file_wins_over_directory_index_for_relative_import() {
    // `./util` with both `util.ts` and `util/index.ts` present: file wins.
    let records = vec![
        rec("src/app.ts", &["./util"], "typescript"),
        rec("src/util.ts", &[], "typescript"),
        rec("src/util/index.ts", &[], "typescript"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        "./util must resolve to util.ts not util/index.ts"
    );
    let app_idx = dg.node_for_path(&PathBuf::from("src/app.ts")).unwrap();
    let util_ts_idx = dg.node_for_path(&PathBuf::from("src/util.ts")).unwrap();
    assert!(
        dg.edges_as_pairs().contains(&(app_idx, util_ts_idx)),
        "edge must go to util.ts (file), not util/index.ts (directory)"
    );
}

// ── Python bare dotted ────────────────────────────────────────────────────────

#[test]
fn python_bare_dotted_resolves_to_py_file() {
    let records = vec![
        rec("src/main.py", &["foo.bar"], "python"),
        rec("foo/bar.py", &[], "python"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(dg.edge_count(), 1, "foo.bar must resolve to foo/bar.py");
}

#[test]
fn python_bare_dotted_resolves_to_init_py() {
    let records = vec![
        rec("src/main.py", &["foo.bar"], "python"),
        rec("foo/bar/__init__.py", &[], "python"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        "foo.bar must resolve to foo/bar/__init__.py"
    );
}

#[test]
fn python_bare_dotted_external_returns_no_edge() {
    // `os` is a stdlib name not in the graph.
    let records = vec![
        rec("src/main.py", &["os"], "python"),
        rec("src/utils.py", &[], "python"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        0,
        "stdlib names must not resolve to in-graph files"
    );
}

// ── Python package-relative ───────────────────────────────────────────────────

#[test]
fn python_relative_single_dot_resolves_to_sibling_module() {
    // `from .sibling import x` → specifier `.sibling` from a/b/c.py → a/b/sibling.py
    let records = vec![
        rec("a/b/c.py", &[".sibling"], "python"),
        rec("a/b/sibling.py", &[], "python"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        ".sibling must resolve to a/b/sibling.py"
    );
}

#[test]
fn python_relative_double_dot_resolves_to_parent_package_init() {
    // `from .. import x` → specifier `..` from a/b/c.py → a/__init__.py
    let records = vec![
        rec("a/b/c.py", &[".."], "python"),
        rec("a/__init__.py", &[], "python"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(dg.edge_count(), 1, ".. must resolve to a/__init__.py");
}

#[test]
fn python_relative_double_dot_with_name_resolves_parent_sibling() {
    // `from ..pkg import x` → specifier `..pkg` from a/b/c.py → a/pkg.py
    let records = vec![
        rec("a/b/c.py", &["..pkg"], "python"),
        rec("a/pkg.py", &[], "python"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(dg.edge_count(), 1, "..pkg must resolve to a/pkg.py");
}

// ── Go module ─────────────────────────────────────────────────────────────────

#[test]
fn go_module_import_resolves_to_internal_package_files() {
    // go.mod: module example.com/myapp
    // import `example.com/myapp/internal/util` → edges to all .go files in internal/util/
    let records = vec![
        rec("cmd/main.go", &["example.com/myapp/internal/util"], "go"),
        rec("internal/util/helper.go", &[], "go"),
        rec("internal/util/types.go", &[], "go"),
    ];
    let dg = build_dependency_graph_with_go_module(&records, Some("example.com/myapp"));
    assert_eq!(
        dg.edge_count(),
        2,
        "one edge per .go file in the imported package"
    );
}

#[test]
fn go_external_import_produces_no_edge() {
    let records = vec![
        rec("cmd/main.go", &["fmt", "os", "github.com/other/repo"], "go"),
        rec("internal/util/helper.go", &[], "go"),
    ];
    let dg = build_dependency_graph_with_go_module(&records, Some("example.com/myapp"));
    assert_eq!(
        dg.edge_count(),
        0,
        "stdlib and foreign packages must not resolve"
    );
}

#[test]
fn go_no_mod_prefix_produces_no_edge() {
    let records = vec![
        rec("cmd/main.go", &["example.com/myapp/internal/util"], "go"),
        rec("internal/util/helper.go", &[], "go"),
    ];
    // No go_module supplied → all Go imports external.
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        0,
        "without go_module prefix all Go imports are external"
    );
}

// ── Java dotted ───────────────────────────────────────────────────────────────

#[test]
fn java_dotted_import_resolves_via_src_main_java() {
    let records = vec![
        rec(
            "src/main/java/com/acme/app/App.java",
            &["com.acme.lib.Util"],
            "java",
        ),
        rec("src/main/java/com/acme/lib/Util.java", &[], "java"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        "com.acme.lib.Util must resolve via src/main/java root"
    );
}

#[test]
fn java_dotted_import_resolves_at_repo_root() {
    // Ad-hoc Java layout without standard Maven structure.
    let records = vec![
        rec("com/acme/app/App.java", &["com.acme.lib.Util"], "java"),
        rec("com/acme/lib/Util.java", &[], "java"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        "com.acme.lib.Util must resolve at repo root"
    );
}

#[test]
fn java_wildcard_import_produces_one_edge_per_class() {
    let records = vec![
        rec(
            "src/main/java/com/acme/app/App.java",
            &["com.acme.lib.*"],
            "java",
        ),
        rec("src/main/java/com/acme/lib/Foo.java", &[], "java"),
        rec("src/main/java/com/acme/lib/Bar.java", &[], "java"),
        rec("src/main/java/com/acme/lib/Baz.java", &[], "java"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        3,
        "wildcard must emit one edge per .java file in package"
    );
}

#[test]
fn java_stdlib_import_produces_no_edge() {
    // java.util.List is a stdlib class, not in path_to_node.
    let records = vec![
        rec("src/Main.java", &["java.util.List"], "java"),
        rec("src/Handler.java", &[], "java"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        0,
        "stdlib Java imports must not produce edges"
    );
}

// ── Rust super:: parent navigation ────────────────────────────────────────────

#[test]
fn super_prefix_walks_to_parent_dir_for_stem_search() {
    // `super::config` from `src/sub/mod.rs` should find `src/config.rs`
    // via subtree search in `src/` (parent of `src/sub/`).
    let records = vec![
        rec("src/sub/mod.rs", &["super::config"], "rust"),
        rec("src/config.rs", &[], "rust"),
    ];
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        "super::config must resolve to src/config.rs"
    );
}
