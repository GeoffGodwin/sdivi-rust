//! Sentinel integration test: the native pipeline NOW populates the `logging` bucket.
//!
//! M33 switches `build_catalog` from `category_for_node_kind` to `classify_hint`.
//! As a result, `call_expression` nodes whose callee text matches the logging regex
//! (e.g. `console.log(...)`, `logger.info(...)`) are now classified as `logging`
//! instead of being classified as `data_access` or dropped.
//!
//! **M33 positive sentinel:** `build_catalog` MUST produce a `logging` bucket when
//! processing TypeScript files that contain matching logging call expressions.
//!
//! **M33 narrowing sentinel:** `data_access` must contain only callee-matching calls
//! (e.g. `fetch(...)`, `db.query(...)`), NOT every call_expression.
//!
//! Related: `queries/mod.rs::category_for_node_kind_never_returns_logging` (M30 sentinel)
//! remains green because it tests `category_for_node_kind` directly — that function
//! is unchanged. Only the pipeline's *choice* of classifier changed in M33.
//!
//! The simple-typescript fixture was extended in M33 to include:
//! - `console.log("Starting run")` → classified as `logging`
//! - `fetch(...)` → classified as `data_access`

use sdivi_config::PatternsConfig;
use sdivi_lang_typescript::TypeScriptAdapter;
use sdivi_parsing::adapter::LanguageAdapter;
use sdivi_parsing::feature_record::FeatureRecord;
use sdivi_patterns::build_catalog;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/sdivi-patterns
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/ parent must exist")
        .parent()
        .expect("workspace root must exist")
        .to_path_buf()
}

fn read_and_parse_ts(rel_path: &str) -> FeatureRecord {
    let abs = workspace_root().join(rel_path);
    let source = std::fs::read_to_string(&abs)
        .unwrap_or_else(|e| panic!("could not read TypeScript fixture {:?}: {e}", abs));
    TypeScriptAdapter.parse_file(Path::new(rel_path), source)
}

fn catalog_config() -> PatternsConfig {
    PatternsConfig {
        min_pattern_nodes: 1,
        ..PatternsConfig::default()
    }
}

/// M33 positive sentinel: `build_catalog` now produces a `logging` bucket for
/// TypeScript files that contain `console.log(...)` calls.
///
/// Pre-M33: `logging` was catalog-only — the native pipeline never emitted it.
/// Post-M33: `classify_hint` routes `call_expression` nodes with matching callee
/// text to `logging`. This test locks in that promotion.
#[test]
fn simple_typescript_fixture_produces_logging_bucket_after_m33() {
    let records = vec![
        read_and_parse_ts("tests/fixtures/simple-typescript/app.ts"),
        read_and_parse_ts("tests/fixtures/simple-typescript/utils.ts"),
        read_and_parse_ts("tests/fixtures/simple-typescript/models.ts"),
    ];

    // Sanity: fixture must have call_expression hints for this test to be non-vacuous.
    let call_expr_count: usize = records
        .iter()
        .flat_map(|r| &r.pattern_hints)
        .filter(|h| h.node_kind == "call_expression")
        .count();
    assert!(
        call_expr_count >= 1,
        "TypeScript fixture must produce at least one call_expression hint; \
         the sentinel test is only meaningful when the native pipeline actively classifies nodes."
    );

    let catalog = build_catalog(&records, &catalog_config());

    // M33 positive: logging bucket MUST now be present (console.log in app.ts).
    // re-baselined in M33: switched to classify_hint
    assert!(
        catalog.entries.contains_key("logging"),
        "build_catalog MUST produce a `logging` bucket in M33 — `classify_hint` routes \
         console.log(...) call_expression nodes to logging. Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );

    let log_total: u32 = catalog.entries["logging"].values().map(|s| s.count).sum();
    assert!(
        log_total >= 1,
        "`logging` bucket must contain at least one instance, got {log_total}"
    );
}

/// M33 narrowing sentinel: `data_access` contains only callee-matching calls
/// (e.g. `fetch(...)`), NOT every call_expression.
///
/// Pre-M33: every call_expression was data_access.
/// Post-M33: only calls matching the data_access regex are classified there.
#[test]
fn simple_typescript_fixture_data_access_requires_matching_callee() {
    let records = vec![
        read_and_parse_ts("tests/fixtures/simple-typescript/app.ts"),
        read_and_parse_ts("tests/fixtures/simple-typescript/utils.ts"),
    ];

    let catalog = build_catalog(&records, &catalog_config());

    // data_access is present (fetch("/api/users/{id}") in app.ts matches).
    // re-baselined in M33: switched to classify_hint — only fetch/db calls match
    assert!(
        catalog.entries.contains_key("data_access"),
        "data_access must be present — fetch(...) in app.ts must match the data_access regex. \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );

    // data_access count must be less than total call_expression count.
    // This proves callee-filtering is in effect — not every call_expression is data_access.
    let total_call_exprs: usize = records
        .iter()
        .flat_map(|r| &r.pattern_hints)
        .filter(|h| h.node_kind == "call_expression")
        .count();
    let da_count: u32 = catalog.entries["data_access"]
        .values()
        .map(|s| s.count)
        .sum();
    assert!(
        (da_count as usize) < total_call_exprs,
        "data_access instance count ({da_count}) must be less than total call_expression \
         count ({total_call_exprs}) — classify_hint drops unrecognised callees"
    );
}
