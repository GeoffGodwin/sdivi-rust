//! M33 integration tests for Go language classification via `build_catalog`.
//!
//! The reviewer flagged a coverage gap: no integration test covers the acceptance
//! criterion "A snapshot run against `tests/fixtures/simple-go` has `data_access`
//! containing only `db.*`/`sql.*`-shape calls and `logging` containing only
//! `fmt.Print*`-shape calls; non-matching calls dropped from both."
//!
//! These tests operate at the `build_catalog` level using synthetic `FeatureRecord`
//! and `PatternHint` structs, covering the full classification path (classify_hint →
//! build_catalog) without requiring the Go language adapter as a test dependency.
//! That level is where M33's behavioural change (call-site swap in catalog.rs) is
//! load-bearing, and it is the appropriate layer for verifying that the Go regexes
//! route correctly through the pipeline.
//!
//! Each test anchors a specific invariant:
//! - `fmt.Println` / `fmt.Printf` → `logging` (GO_RE matches)
//! - `db.query` / `sql.Open` → `data_access` (TS_JS_GO_RE matches)
//! - `os.Exit` / `strings.Join` → dropped (no regex match, empty Vec)
//! - Mixed fixture → all three paths exercised, category totals are consistent

use sdivi_config::PatternsConfig;
use sdivi_parsing::feature_record::{FeatureRecord, PatternHint};
use sdivi_patterns::build_catalog;
use std::path::PathBuf;

// ── helpers ───────────────────────────────────────────────────────────────────

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
        imports: vec!["fmt".to_string()],
        exports: vec![],
        signatures: vec![],
        pattern_hints: hints,
    }
}

fn catalog_config_min1() -> PatternsConfig {
    PatternsConfig {
        min_pattern_nodes: 1,
        ..PatternsConfig::default()
    }
}

// ── single-callee routing tests ───────────────────────────────────────────────

/// M33 acceptance criterion: `fmt.Println(...)` call_expression routes to `logging`.
///
/// Pre-M33: every call_expression was `data_access` regardless of callee.
/// Post-M33: only calls matching the data_access regex land there; fmt.Print* goes to logging.
///
/// re-baselined in M33: switched to classify_hint
#[test]
fn go_fmt_println_routes_to_logging_in_catalog() {
    let records = vec![go_record(
        "main.go",
        vec![go_hint("call_expression", "fmt.Println(os.Args)", 10)],
    )];

    let catalog = build_catalog(&records, &catalog_config_min1());

    assert!(
        catalog.entries.contains_key("logging"),
        "build_catalog must produce a `logging` bucket for Go fmt.Println calls (M33). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );

    let log_total: u32 = catalog.entries["logging"].values().map(|s| s.count).sum();
    assert_eq!(
        log_total, 1,
        "logging bucket must contain exactly 1 instance for fmt.Println, got {log_total}"
    );

    // fmt.Println must NOT be classified as data_access (pre-M33 behaviour, now gone).
    assert!(
        !catalog.entries.contains_key("data_access"),
        "fmt.Println must NOT produce a data_access entry in M33; it routes to logging. \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
}

/// M33 acceptance criterion: `fmt.Printf(...)` call_expression routes to `logging`.
///
/// GO_RE matches `^fmt\.(Print|Println|Printf|Errorf|Fprint|Sprint)`.
#[test]
fn go_fmt_printf_routes_to_logging_in_catalog() {
    let records = vec![go_record(
        "main.go",
        vec![go_hint("call_expression", "fmt.Printf(\"%v\", x)", 5)],
    )];

    let catalog = build_catalog(&records, &catalog_config_min1());

    assert!(
        catalog.entries.contains_key("logging"),
        "build_catalog must produce a `logging` bucket for Go fmt.Printf calls (M33). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    assert!(
        !catalog.entries.contains_key("data_access"),
        "fmt.Printf must NOT produce a data_access entry — it routes to logging via GO_RE"
    );
}

/// M33 acceptance criterion: `db.query(sql)` call_expression routes to `data_access`.
///
/// TS_JS_GO_RE matches `\b(db|sql)\.` so `db.query` and `sql.Open` both match.
#[test]
fn go_db_query_routes_to_data_access_in_catalog() {
    let records = vec![go_record(
        "repo.go",
        vec![go_hint("call_expression", "db.query(sql)", 20)],
    )];

    let catalog = build_catalog(&records, &catalog_config_min1());

    assert!(
        catalog.entries.contains_key("data_access"),
        "build_catalog must produce a `data_access` bucket for Go db.query calls (M33). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );

    let da_total: u32 = catalog.entries["data_access"].values().map(|s| s.count).sum();
    assert_eq!(
        da_total, 1,
        "data_access bucket must contain exactly 1 instance for db.query, got {da_total}"
    );

    // db.query must NOT be classified as logging.
    assert!(
        !catalog.entries.contains_key("logging"),
        "db.query must NOT produce a logging entry — it routes to data_access via TS_JS_GO_RE"
    );
}

/// M33 acceptance criterion: non-matching Go calls are dropped (silently, same as prior None path).
///
/// `os.Exit(1)` does not match any logging, data_access, or async_patterns regex for Go.
/// The hint must be silently dropped — neither `logging` nor `data_access` appears.
#[test]
fn go_nonmatching_call_is_dropped_from_catalog() {
    let records = vec![go_record(
        "main.go",
        vec![go_hint("call_expression", "os.Exit(1)", 3)],
    )];

    let catalog = build_catalog(&records, &catalog_config_min1());

    assert!(
        !catalog.entries.contains_key("logging"),
        "os.Exit(1) must NOT appear in logging — it matches no logging regex for Go. \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    assert!(
        !catalog.entries.contains_key("data_access"),
        "os.Exit(1) must NOT appear in data_access — it matches no data_access regex for Go. \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    assert!(
        catalog.entries.is_empty(),
        "os.Exit(1) should produce an empty catalog (dropped hint). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
}

/// Non-matching: `strings.Join(...)` is a commonly-called Go stdlib function with no category.
#[test]
fn go_strings_join_is_dropped_from_catalog() {
    let records = vec![go_record(
        "util.go",
        vec![go_hint("call_expression", "strings.Join(parts, \", \")", 7)],
    )];

    let catalog = build_catalog(&records, &catalog_config_min1());

    assert!(
        catalog.entries.is_empty(),
        "strings.Join must produce an empty catalog (no matching regex for Go). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
}

// ── mixed-callee integration test (the core acceptance criterion) ─────────────

/// M33 acceptance criterion — full: Go catalog correctly routes fmt.Print* to logging,
/// db.* to data_access, and drops non-matching calls; all three paths in one record.
///
/// This is the load-bearing test for the coverage gap flagged by the reviewer.
/// It verifies that `build_catalog` now uses `classify_hint` (M33) rather than
/// `category_for_node_kind` (pre-M33) for Go call_expression nodes.
///
/// re-baselined in M33: switched to classify_hint — logging and data_access
/// buckets are now populated separately; non-matching calls dropped.
#[test]
fn go_mixed_calls_route_correctly_by_callee_in_catalog() {
    let records = vec![go_record(
        "app.go",
        vec![
            // fmt.Println → logging (matches GO_RE: ^fmt\.Println)
            go_hint("call_expression", "fmt.Println(\"handled:\", event)", 10),
            // fmt.Printf → logging (matches GO_RE: ^fmt\.Printf)
            go_hint("call_expression", "fmt.Printf(\"%v\", x)", 11),
            // db.Query → data_access (matches TS_JS_GO_RE: \b(db|sql)\.)
            go_hint("call_expression", "db.Query(ctx, sql)", 15),
            // sql.Open → data_access (matches TS_JS_GO_RE: \b(db|sql)\.)
            go_hint("call_expression", "sql.Open(\"postgres\", dsn)", 16),
            // os.Exit → dropped (no match)
            go_hint("call_expression", "os.Exit(1)", 20),
            // strings.Join → dropped (no match)
            go_hint("call_expression", "strings.Join(parts, \", \")", 21),
        ],
    )];

    let catalog = build_catalog(&records, &catalog_config_min1());

    // logging must be present — fmt.Println and fmt.Printf both match GO_RE.
    assert!(
        catalog.entries.contains_key("logging"),
        "logging bucket must be present for Go fmt.Print* calls (M33 acceptance criterion). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );

    // data_access must be present — db.Query and sql.Open both match TS_JS_GO_RE.
    assert!(
        catalog.entries.contains_key("data_access"),
        "data_access bucket must be present for Go db.*/sql.* calls (M33 acceptance criterion). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );

    let log_total: u32 = catalog.entries["logging"].values().map(|s| s.count).sum();
    let da_total: u32 = catalog.entries["data_access"].values().map(|s| s.count).sum();

    // 2 fmt.Print* calls → 2 logging instances
    assert_eq!(
        log_total, 2,
        "logging must contain 2 instances (fmt.Println + fmt.Printf), got {log_total}"
    );

    // 2 db.*/sql.* calls → 2 data_access instances
    assert_eq!(
        da_total, 2,
        "data_access must contain 2 instances (db.Query + sql.Open), got {da_total}"
    );

    // Total classified = 4 out of 6 hints; 2 dropped (os.Exit, strings.Join).
    // This proves the "non-matching calls dropped" part of the acceptance criterion.
    let total_classified = log_total + da_total;
    assert_eq!(
        total_classified, 4,
        "total classified instances must be 4 (2 logging + 2 data_access); \
         os.Exit and strings.Join must be dropped. Got {total_classified}."
    );
}

// ── additional boundary cases ─────────────────────────────────────────────────

/// `fmt.Errorf` is in GO_RE and must route to logging (not data_access).
///
/// Pre-M33: fmt.Errorf was `data_access` (it's a call_expression).
/// Post-M33: fmt.Errorf matches GO_RE `^fmt\.Errorf` and routes to `logging`.
#[test]
fn go_fmt_errorf_routes_to_logging_not_data_access() {
    let records = vec![go_record(
        "err.go",
        vec![go_hint("call_expression", "fmt.Errorf(\"bad: %w\", err)", 5)],
    )];

    let catalog = build_catalog(&records, &catalog_config_min1());

    assert!(
        catalog.entries.contains_key("logging"),
        "fmt.Errorf must route to logging (matches GO_RE ^fmt\\.Errorf). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    assert!(
        !catalog.entries.contains_key("data_access"),
        "fmt.Errorf must NOT appear in data_access (M33 routes it to logging). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
}

/// `sql.Open(...)` routes to data_access — TS_JS_GO_RE matches `\b(db|sql)\.`.
///
/// This satisfies the acceptance criterion "data_access containing only db.*/sql.* calls."
#[test]
fn go_sql_open_routes_to_data_access_in_catalog() {
    let records = vec![go_record(
        "db.go",
        vec![go_hint("call_expression", "sql.Open(\"postgres\", dsn)", 3)],
    )];

    let catalog = build_catalog(&records, &catalog_config_min1());

    assert!(
        catalog.entries.contains_key("data_access"),
        "sql.Open must produce a data_access entry (matches TS_JS_GO_RE \\b(db|sql)\\.). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    assert!(
        !catalog.entries.contains_key("logging"),
        "sql.Open must NOT appear in logging. \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
}

/// Verify the min_pattern_nodes filter works correctly for Go records.
///
/// When min_pattern_nodes = 2 and there is only 1 fmt.Println hint, logging must
/// be absent (filtered out by the count threshold).
#[test]
fn go_min_pattern_nodes_filter_drops_single_instance_logging() {
    let records = vec![go_record(
        "main.go",
        vec![go_hint("call_expression", "fmt.Println(\"x\")", 1)],
    )];

    let config = PatternsConfig {
        min_pattern_nodes: 2, // threshold: need at least 2
        ..PatternsConfig::default()
    };
    let catalog = build_catalog(&records, &config);

    // Only 1 instance of fmt.Println — below the min_pattern_nodes=2 threshold.
    assert!(
        !catalog.entries.contains_key("logging"),
        "logging must be absent when the only fmt.Println instance count (1) < min_pattern_nodes (2). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
}
