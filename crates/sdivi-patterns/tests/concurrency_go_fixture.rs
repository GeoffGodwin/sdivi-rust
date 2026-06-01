//! M44 integration tests: Go goroutine and select detection via `build_catalog`.
//!
//! Verifies that `go_statement` and `select_statement` nodes — already collected
//! by the Go adapter — are now routed to the `concurrency` catalog bucket.
//! Uses synthetic `FeatureRecord` + `PatternHint` structs so the Go adapter is
//! not required as a test dependency.

use sdivi_config::PatternsConfig;
use sdivi_parsing::feature_record::{FeatureRecord, PatternHint};
use sdivi_patterns::build_catalog;
use std::path::PathBuf;

fn go_hint(node_kind: &str, text: &str, row: usize) -> PatternHint {
    PatternHint {
        node_kind: node_kind.to_string(),
        start_byte: 0,
        end_byte: text.len(),
        start_row: row,
        start_col: 0,
        text: text.to_string(),
    }
}

fn go_record(file: &str, hints: Vec<PatternHint>) -> FeatureRecord {
    FeatureRecord {
        path: PathBuf::from(file),
        language: "go".to_string(),
        imports: vec![],
        exports: vec![],
        signatures: vec![],
        pattern_hints: hints,
    }
}

fn catalog_min1() -> PatternsConfig {
    PatternsConfig {
        min_pattern_nodes: 1,
        ..PatternsConfig::default()
    }
}

/// M44 acceptance criterion: a Go record with a goroutine yields a `concurrency` bucket.
#[test]
fn go_statement_routes_to_concurrency_bucket() {
    let records = vec![go_record(
        "worker.go",
        vec![go_hint("go_statement", "go worker(ch)", 10)],
    )];
    let catalog = build_catalog(&records, &catalog_min1());
    assert!(
        catalog.entries.contains_key("concurrency"),
        "build_catalog must produce a `concurrency` bucket for go_statement hints (M44). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    let total: u32 = catalog.entries["concurrency"]
        .values()
        .map(|s| s.count)
        .sum();
    assert_eq!(
        total, 1,
        "concurrency bucket must contain 1 go_statement instance"
    );
}

/// M44 acceptance criterion: a Go record with a select statement yields a `concurrency` bucket.
#[test]
fn select_statement_routes_to_concurrency_bucket() {
    let records = vec![go_record(
        "main.go",
        vec![go_hint(
            "select_statement",
            "select { case msg := <-ch: handle(msg) }",
            20,
        )],
    )];
    let catalog = build_catalog(&records, &catalog_min1());
    assert!(
        catalog.entries.contains_key("concurrency"),
        "build_catalog must produce a `concurrency` bucket for select_statement hints (M44). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
}

/// A fixture with both a goroutine and a select statement yields combined `concurrency` counts.
#[test]
fn go_goroutine_plus_select_yields_concurrency_instances() {
    let records = vec![go_record(
        "server.go",
        vec![
            go_hint("go_statement", "go worker(ch)", 5),
            go_hint("go_statement", "go monitor(done)", 10),
            go_hint("select_statement", "select { case <-done: return }", 15),
        ],
    )];
    let catalog = build_catalog(&records, &catalog_min1());
    assert!(
        catalog.entries.contains_key("concurrency"),
        "concurrency bucket must be present for goroutines + select (M44 acceptance criterion). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    let total: u32 = catalog.entries["concurrency"]
        .values()
        .map(|s| s.count)
        .sum();
    assert_eq!(
        total, 3,
        "concurrency bucket must contain 3 instances (2 go_statement + 1 select_statement)"
    );
}

/// `defer_statement` must NOT route to concurrency — it belongs to resource_management (M45.1).
#[test]
fn defer_statement_does_not_route_to_concurrency() {
    let records = vec![go_record(
        "cleanup.go",
        vec![go_hint("defer_statement", "defer cleanup()", 5)],
    )];
    let catalog = build_catalog(&records, &catalog_min1());
    assert!(
        !catalog.entries.contains_key("concurrency"),
        "defer_statement must NOT produce a `concurrency` entry. \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
}
