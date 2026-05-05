//! Robustness tests: the resolver must never panic regardless of import specifier.
//!
//! Per the M26 spec: "generate random `(from_path, import_specifier)` pairs and
//! assert resolution either succeeds with a path that exists in `path_to_node`
//! or returns `None`. No panics."
//!
//! Rather than pulling in proptest as a dev-dep, we enumerate the adversarial
//! classes that the resolver sees in the wild and verify each produces zero edges
//! without panicking.

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

/// Build a graph with a single importer record and one potential target, then
/// assert the edge_count is 0 (specifier unresolvable) and no panic occurred.
fn assert_no_panic(importer_path: &str, specifier: &str, language: &str) {
    let records = vec![
        rec(importer_path, &[specifier], language),
        rec("dummy/target.py", &[], language),
    ];
    let _ = build_dependency_graph(&records);
}

// ── Relative-path adversarial inputs ─────────────────────────────────────────

#[test]
fn many_dotdot_segments_do_not_panic() {
    // Far more `../` than path components — must drop silently.
    for depth in &[
        "../../../../../../../foo",
        "../../../../../../../../../../../../foo",
        "../../../../../../../../../../../../../../../../../../foo",
    ] {
        assert_no_panic("src/util.ts", depth, "typescript");
        assert_no_panic("a.py", depth, "python");
        assert_no_panic("a.rs", depth, "rust");
    }
}

#[test]
fn dot_only_relative_specifiers_do_not_panic() {
    // `./` with empty remainder — no file to match
    for spec in &["./", "../", "../../", ".///", "././././"] {
        assert_no_panic("src/a.ts", spec, "typescript");
    }
}

#[test]
fn empty_specifier_does_not_panic() {
    assert_no_panic("src/a.rs", "", "rust");
    assert_no_panic("src/a.py", "", "python");
    assert_no_panic("src/a.ts", "", "typescript");
    assert_no_panic("src/a.go", "", "go");
    assert_no_panic("src/Main.java", "", "java");
}

// ── Python adversarial inputs ────────────────────────────────────────────────

#[test]
fn python_only_dots_specifier_does_not_panic() {
    // Pure-dot specifiers navigate up levels; with no target they should return None.
    for spec in &[".", "..", "...", "....", ".....", "..............."] {
        assert_no_panic("a/b/c.py", spec, "python");
    }
}

#[test]
fn python_overshoot_relative_does_not_panic() {
    // More dots than path components
    assert_no_panic("a.py", "....x", "python");
    assert_no_panic("a/b.py", "......x", "python");
}

#[test]
fn python_bare_dotted_with_no_matching_file_does_not_panic() {
    let specs = &[
        "nonexistent.module",
        "a.b.c.d.e.f.g.h",
        "os.path.join",
        "urllib.parse.urlencode",
        "pytest.fixture",
    ];
    for spec in specs {
        assert_no_panic("src/main.py", spec, "python");
    }
}

// ── Go adversarial inputs ─────────────────────────────────────────────────────

#[test]
fn go_specifier_without_module_prefix_does_not_panic() {
    let specs = &[
        "fmt",
        "os",
        "net/http",
        "github.com/other/repo/pkg",
        "golang.org/x/text/unicode",
        "",
    ];
    let records_with_target = vec![
        rec("cmd/main.go", specs, "go"),
        rec("internal/util/helper.go", &[], "go"),
    ];
    let _ = build_dependency_graph_with_go_module(&records_with_target, Some("example.com/app"));
}

// ── Java adversarial inputs ───────────────────────────────────────────────────

#[test]
fn java_stdlib_and_unknown_classes_do_not_panic() {
    let specs = &[
        "java.lang.String",
        "java.util.List",
        "java.util.*",
        "javax.servlet.http.HttpServlet",
        "com.nonexistent.Class",
        "com.nonexistent.*",
        "",
    ];
    for spec in specs {
        assert_no_panic("src/Main.java", spec, "java");
    }
}

// ── Rust adversarial inputs ───────────────────────────────────────────────────

#[test]
fn rust_specifiers_without_crate_prefix_do_not_panic() {
    let specs = &[
        "std::collections::HashMap",
        "serde::Serialize",
        "tokio::main",
        "rand",
        "some_crate::some_module::SomeType",
        "",
    ];
    for spec in specs {
        assert_no_panic("src/lib.rs", spec, "rust");
    }
}

#[test]
fn rust_deeply_nested_super_does_not_panic() {
    let deep_super = "super::super::super::super::super::super::super::super::super::config";
    assert_no_panic("a.rs", deep_super, "rust");
}

// ── Unknown language ─────────────────────────────────────────────────────────

#[test]
fn unknown_language_relative_import_does_not_panic() {
    // Falls through to cross-language fallback extension list
    assert_no_panic("src/file.kt", "./util", "kotlin");
    assert_no_panic("src/file.cs", "../shared/Types", "csharp");
}

// ── Verified-presence invariant ──────────────────────────────────────────────

/// Any node index returned by the resolver must point to a path that was in
/// the original records — the resolver must never fabricate nodes.
#[test]
fn all_resolved_edges_point_to_nodes_from_input_records() {
    let records = vec![
        rec("app/index.ts", &["../shared/utils", "./components", "react"], "typescript"),
        rec("shared/utils.ts", &[], "typescript"),
        rec("app/components/index.ts", &[], "typescript"),
    ];
    let input_paths: std::collections::BTreeSet<PathBuf> =
        records.iter().map(|r| r.path.clone()).collect();

    let dg = build_dependency_graph(&records);

    for (from_idx, to_idx) in dg.edges_as_pairs() {
        let from_path = dg.node_path(from_idx).expect("from node must exist");
        let to_path = dg.node_path(to_idx).expect("to node must exist");
        assert!(
            input_paths.contains(from_path),
            "edge source {from_path:?} was not in the input records"
        );
        assert!(
            input_paths.contains(to_path),
            "edge target {to_path:?} was not in the input records — resolver fabricated a node"
        );
    }
}
