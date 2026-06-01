//! M45.1 integration tests: Python/Go/Java resource_management detection via `build_catalog`.
//!
//! Verifies that `with_statement`, `defer_statement`, and `try_with_resources_statement`
//! nodes — already collected by their respective adapters but previously dropped —
//! are now routed to the `resource_management` catalog bucket.
//! Uses synthetic `FeatureRecord` + `PatternHint` structs.

use sdivi_config::PatternsConfig;
use sdivi_parsing::feature_record::{FeatureRecord, PatternHint};
use sdivi_patterns::build_catalog;
use std::path::PathBuf;

fn make_hint(node_kind: &str, text: &str, row: usize) -> PatternHint {
    PatternHint {
        node_kind: node_kind.to_string(),
        start_byte: 0,
        end_byte: text.len(),
        start_row: row,
        start_col: 0,
        text: text.to_string(),
    }
}

fn make_record(file: &str, language: &str, hints: Vec<PatternHint>) -> FeatureRecord {
    FeatureRecord {
        path: PathBuf::from(file),
        language: language.to_string(),
        imports: vec![],
        exports: vec![],
        signatures: vec![],
        pattern_hints: hints,
    }
}

fn min1_config() -> PatternsConfig {
    PatternsConfig {
        min_pattern_nodes: 1,
        ..PatternsConfig::default()
    }
}

/// M45.1: Python `with_statement` routes to resource_management.
#[test]
fn python_with_statement_routes_to_resource_management() {
    let records = vec![make_record(
        "io.py",
        "python",
        vec![make_hint("with_statement", "with open(path) as f:", 5)],
    )];
    let catalog = build_catalog(&records, &min1_config());
    assert!(
        catalog.entries.contains_key("resource_management"),
        "build_catalog must produce a `resource_management` bucket for with_statement (M45.1). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    let total: u32 = catalog.entries["resource_management"]
        .values()
        .map(|s| s.count)
        .sum();
    assert_eq!(
        total, 1,
        "resource_management bucket must contain 1 with_statement"
    );
}

/// M45.1: Go `defer_statement` routes to resource_management.
#[test]
fn go_defer_statement_routes_to_resource_management() {
    let records = vec![make_record(
        "cleanup.go",
        "go",
        vec![make_hint("defer_statement", "defer f.Close()", 10)],
    )];
    let catalog = build_catalog(&records, &min1_config());
    assert!(
        catalog.entries.contains_key("resource_management"),
        "build_catalog must produce a `resource_management` bucket for defer_statement (M45.1). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    let total: u32 = catalog.entries["resource_management"]
        .values()
        .map(|s| s.count)
        .sum();
    assert_eq!(
        total, 1,
        "resource_management bucket must contain 1 defer_statement"
    );
}

/// M45.1: Java `try_with_resources_statement` routes to resource_management.
#[test]
fn java_try_with_resources_routes_to_resource_management() {
    let records = vec![make_record(
        "FileUtil.java",
        "java",
        vec![make_hint(
            "try_with_resources_statement",
            "try (var r = open(p)) { }",
            20,
        )],
    )];
    let catalog = build_catalog(&records, &min1_config());
    assert!(
        catalog.entries.contains_key("resource_management"),
        "build_catalog must produce a `resource_management` bucket for \
         try_with_resources_statement (M45.1). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    let total: u32 = catalog.entries["resource_management"]
        .values()
        .map(|s| s.count)
        .sum();
    assert_eq!(
        total, 1,
        "resource_management bucket must contain 1 try_with_resources_statement"
    );
}

/// Multiple resource-management patterns across languages are counted correctly.
#[test]
fn mixed_language_resource_management_counts() {
    let records = vec![
        make_record(
            "io.py",
            "python",
            vec![
                make_hint("with_statement", "with open(p) as f:", 5),
                make_hint("with_statement", "with lock:", 10),
            ],
        ),
        make_record(
            "server.go",
            "go",
            vec![
                make_hint("defer_statement", "defer f.Close()", 3),
                make_hint("defer_statement", "defer mu.Unlock()", 8),
                make_hint("defer_statement", "defer wg.Done()", 15),
            ],
        ),
        make_record(
            "Util.java",
            "java",
            vec![make_hint(
                "try_with_resources_statement",
                "try (var r = open(p)) { }",
                7,
            )],
        ),
    ];
    let catalog = build_catalog(&records, &min1_config());
    assert!(
        catalog.entries.contains_key("resource_management"),
        "resource_management bucket must be present for mixed-language fixture"
    );
    let total: u32 = catalog.entries["resource_management"]
        .values()
        .map(|s| s.count)
        .sum();
    assert_eq!(
        total, 6,
        "resource_management must count 2 Python + 3 Go + 1 Java = 6 instances"
    );
}

/// Go `defer_statement` does NOT route to concurrency after M45.1.
#[test]
fn defer_statement_does_not_appear_in_concurrency_after_m45_1() {
    let records = vec![make_record(
        "cleanup.go",
        "go",
        vec![make_hint("defer_statement", "defer mu.Unlock()", 5)],
    )];
    let catalog = build_catalog(&records, &min1_config());
    assert!(
        !catalog.entries.contains_key("concurrency"),
        "defer_statement must NOT produce a `concurrency` entry after M45.1. \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
}
