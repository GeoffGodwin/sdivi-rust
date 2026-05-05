//! Multi-language integration test for the M26 resolver.
//!
//! Builds a synthetic repository spanning TypeScript, Python, Go, and Java
//! and asserts that each language's resolver produces the expected edge count.

use sdivi_graph::dependency_graph::build_dependency_graph_with_go_module;
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

/// Synthetic multi-language repository:
///
/// TypeScript (3 edges):
///   app/index.ts → ../shared/utils.ts (parent nav)
///   app/index.ts → ./components/button (directory index)
///   app/components/button/index.ts → ../icon.ts (parent nav)
///
/// Python (2 edges):
///   pkg/app.py → .models (relative) → pkg/models.py
///   pkg/app.py → util.helper (bare dotted) → util/helper.py
///
/// Go (2 edges):
///   cmd/main.go → mymod/internal/db → 2 .go files
///
/// Java (1 edge):
///   src/main/java/com/app/Main.java → com.lib.Util
#[test]
fn multi_language_synthetic_repo_produces_expected_edges() {
    let records = vec![
        // TypeScript
        rec(
            "app/index.ts",
            &["../shared/utils", "./components/button"],
            "typescript",
        ),
        rec("shared/utils.ts", &[], "typescript"),
        rec("app/components/button/index.ts", &["../icon"], "typescript"),
        rec("app/components/icon.ts", &[], "typescript"),
        // Python
        rec("pkg/app.py", &[".models", "util.helper"], "python"),
        rec("pkg/models.py", &[], "python"),
        rec("util/helper.py", &[], "python"),
        // Go
        rec("cmd/main.go", &["mymod/internal/db"], "go"),
        rec("internal/db/conn.go", &[], "go"),
        rec("internal/db/query.go", &[], "go"),
        // Java
        rec("src/main/java/com/app/Main.java", &["com.lib.Util"], "java"),
        rec("src/main/java/com/lib/Util.java", &[], "java"),
    ];

    let dg = build_dependency_graph_with_go_module(&records, Some("mymod"));

    // Total expected: 3 TS + 2 Python + 2 Go + 1 Java = 8
    assert_eq!(
        dg.edge_count(),
        8,
        "multi-language repo must produce exactly 8 edges post-M26 \
         (3 TS + 2 Python + 2 Go + 1 Java)"
    );

    // Spot-check per-language contributions.
    let ts_index_idx = dg
        .node_for_path(&PathBuf::from("app/index.ts"))
        .expect("app/index.ts must be in graph");
    let shared_utils_idx = dg
        .node_for_path(&PathBuf::from("shared/utils.ts"))
        .expect("shared/utils.ts must be in graph");
    assert!(
        dg.edges_as_pairs()
            .contains(&(ts_index_idx, shared_utils_idx)),
        "TS parent-nav edge app/index.ts → shared/utils.ts must exist"
    );

    let go_main_idx = dg
        .node_for_path(&PathBuf::from("cmd/main.go"))
        .expect("cmd/main.go must be in graph");
    let go_edge_targets: Vec<usize> = dg
        .edges_as_pairs()
        .into_iter()
        .filter(|(from, _)| *from == go_main_idx)
        .map(|(_, to)| to)
        .collect();
    assert_eq!(
        go_edge_targets.len(),
        2,
        "Go import must emit one edge per .go file in package"
    );
}
