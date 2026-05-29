//! Fixture-level integration tests: `data_access` pattern bucket from real source files.
//!
//! Verifies the M29 acceptance criterion:
//! "A snapshot run against tests/fixtures/simple-typescript produces a non-empty
//! data_access bucket in pattern_metrics; a snapshot against tests/fixtures/simple-python
//! likewise."
//!
//! Tests operate at the `build_catalog` layer — the layer where node-kind classification
//! into pattern categories happens — using real fixture files parsed by the real language
//! adapters. This is equivalent to the pipeline's pattern stage without the I/O overhead
//! of a full `Pipeline::snapshot` call.

use sdivi_config::PatternsConfig;
use sdivi_lang_python::PythonAdapter;
use sdivi_lang_typescript::TypeScriptAdapter;
use sdivi_parsing::adapter::LanguageAdapter;
use sdivi_parsing::feature_record::FeatureRecord;
use sdivi_patterns::build_catalog;
use sdivi_patterns::queries::category_for_node_kind;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/sdivi-patterns
    // parent()           = crates/
    // parent()           = workspace root
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

fn read_and_parse_py(rel_path: &str) -> FeatureRecord {
    let abs = workspace_root().join(rel_path);
    let source = std::fs::read_to_string(&abs)
        .unwrap_or_else(|e| panic!("could not read Python fixture {:?}: {e}", abs));
    PythonAdapter.parse_file(Path::new(rel_path), source)
}

fn catalog_config() -> PatternsConfig {
    PatternsConfig {
        min_pattern_nodes: 1,
        ..PatternsConfig::default()
    }
}

/// TypeScript fixture files contain `call_expression` nodes (e.g., `helper('/tmp')`,
/// `path.replace(...)`). After M29, those node kinds map to `data_access` in the
/// pattern catalog. Verifies the primary observable behavior of the milestone.
#[test]
fn simple_typescript_fixture_produces_data_access_bucket() {
    let records = vec![
        read_and_parse_ts("tests/fixtures/simple-typescript/app.ts"),
        read_and_parse_ts("tests/fixtures/simple-typescript/utils.ts"),
    ];

    // Sanity: at least one record must carry call_expression hints.
    let call_expr_count: usize = records
        .iter()
        .flat_map(|r| &r.pattern_hints)
        .filter(|h| h.node_kind == "call_expression")
        .count();
    assert!(
        call_expr_count >= 1,
        "TypeScript fixture must produce at least one call_expression hint; got 0. \
         Check that the fixture files contain function calls."
    );

    let catalog = build_catalog(&records, &catalog_config());
    assert!(
        catalog.entries.contains_key("data_access"),
        "PatternCatalog must include a `data_access` bucket for TypeScript fixtures \
         containing call_expression nodes; present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );

    let da = &catalog.entries["data_access"];
    let total: u32 = da.values().map(|s| s.count).sum();
    assert!(
        total >= 1,
        "`data_access` bucket must contain at least one pattern instance, got {total}"
    );
}

/// Python fixture files contain `call` nodes (e.g., `os.getcwd()`, `User(...)`,
/// `helper(path)`, `re.sub(...)`). After M29, `"call"` was added to the Python
/// adapter's PATTERN_KINDS and those nodes map to `data_access` in the catalog.
#[test]
fn simple_python_fixture_produces_data_access_bucket() {
    let records = vec![
        read_and_parse_py("tests/fixtures/simple-python/main.py"),
        read_and_parse_py("tests/fixtures/simple-python/utils.py"),
    ];

    // Sanity: at least one record must carry call hints.
    let call_count: usize = records
        .iter()
        .flat_map(|r| &r.pattern_hints)
        .filter(|h| h.node_kind == "call")
        .count();
    assert!(
        call_count >= 1,
        "Python fixture must produce at least one `call` hint after M29 adds \
         \"call\" to PATTERN_KINDS; got 0. Verify the Python adapter includes \"call\"."
    );

    let catalog = build_catalog(&records, &catalog_config());
    assert!(
        catalog.entries.contains_key("data_access"),
        "PatternCatalog must include a `data_access` bucket for Python fixtures \
         containing call nodes; present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );

    let da = &catalog.entries["data_access"];
    let total: u32 = da.values().map(|s| s.count).sum();
    assert!(
        total >= 1,
        "`data_access` bucket must contain at least one pattern instance, got {total}"
    );
}

/// Optional Go insurance: Go also uses `call_expression` (same node kind as
/// TypeScript/JavaScript). The classification must hold regardless of the
/// language tag, since `category_for_node_kind` ignores `_language` in v0.
#[test]
fn call_expression_maps_to_data_access_for_go() {
    assert_eq!(
        category_for_node_kind("call_expression", "go"),
        Some("data_access"),
        "call_expression must map to data_access for Go (same as TypeScript/JavaScript)"
    );
}
