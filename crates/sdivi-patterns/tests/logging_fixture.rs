//! Sentinel integration test: the native pipeline NEVER emits a `logging` bucket.
//!
//! M30 adds `logging` as a catalog-only category. The native pattern stage —
//! `build_catalog` — must not produce a `logging` bucket even when processing
//! TypeScript source files that contain `call_expression` nodes, because those
//! node kinds are routed to `data_access`, not `logging`.
//!
//! Foreign extractors (e.g. Meridian) are the only path that produces `logging`
//! entries; they do so by passing `PatternInstanceInput { category: "logging", … }`
//! directly to `compute_pattern_metrics`, bypassing `build_catalog` entirely.
//!
//! This test fills the coverage gap noted in the M30 Reviewer Report:
//! "No integration test runs the pattern stage against tests/fixtures/simple-typescript
//! and asserts zero logging keys in the catalog."

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

/// The native pattern stage must never produce a `logging` bucket, even for
/// TypeScript files that contain `call_expression` nodes — those nodes route to
/// `data_access`. The `logging` category is catalog-only; only foreign extractors
/// that apply callee-text filtering should emit it.
#[test]
fn simple_typescript_fixture_produces_no_logging_bucket() {
    let records = vec![
        read_and_parse_ts("tests/fixtures/simple-typescript/app.ts"),
        read_and_parse_ts("tests/fixtures/simple-typescript/utils.ts"),
        read_and_parse_ts("tests/fixtures/simple-typescript/models.ts"),
    ];

    // Sanity: the fixture must contain call_expression nodes so we know the
    // pattern stage is actually classifying something — we're not vacuously
    // asserting on an empty catalog.
    let call_expr_count: usize = records
        .iter()
        .flat_map(|r| &r.pattern_hints)
        .filter(|h| h.node_kind == "call_expression")
        .count();
    assert!(
        call_expr_count >= 1,
        "TypeScript fixture must produce at least one call_expression hint; \
         the sentinel test is only meaningful when the native pipeline is \
         actively classifying nodes."
    );

    let catalog = build_catalog(&records, &catalog_config());

    assert!(
        !catalog.entries.contains_key("logging"),
        "build_catalog must NEVER produce a `logging` bucket — the category is \
         catalog-only in v0 and can only be populated by foreign extractors \
         supplying PatternInstanceInput directly. Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );

    // The call_expression nodes must still be classified as data_access.
    assert!(
        catalog.entries.contains_key("data_access"),
        "call_expression nodes must still route to `data_access`, not be dropped; \
         present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
}
