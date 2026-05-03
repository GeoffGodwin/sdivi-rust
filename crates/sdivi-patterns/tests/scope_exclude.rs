//! Verifies that `scope_exclude` removes files from the catalog without
//! removing them from the underlying `FeatureRecord` stream.

use std::path::PathBuf;

use sdivi_config::PatternsConfig;
use sdivi_parsing::feature_record::{FeatureRecord, PatternHint};
use sdivi_patterns::build_catalog;

fn hint(node_kind: &str) -> PatternHint {
    PatternHint {
        node_kind: node_kind.to_string(),
        start_byte: 0,
        end_byte: 20,
        start_row: 0,
        start_col: 0,
        text: "stub".to_string(),
    }
}

fn record(path: &str, hints: Vec<PatternHint>) -> FeatureRecord {
    FeatureRecord {
        path: PathBuf::from(path),
        language: "rust".to_string(),
        imports: vec![],
        exports: vec![],
        signatures: vec![],
        pattern_hints: hints,
    }
}

/// File in `scope_exclude` is absent from catalog, present in FeatureRecord slice.
#[test]
fn excluded_file_absent_from_catalog_present_in_records() {
    let vendor_hints = vec![hint("try_expression"), hint("try_expression")];
    let normal_hints = vec![hint("try_expression"), hint("try_expression")];

    let records = vec![
        record("src/vendor/generated.rs", vendor_hints),
        record("src/lib.rs", normal_hints),
    ];

    let config = PatternsConfig {
        min_pattern_nodes: 1,
        scope_exclude: vec!["src/vendor/**".to_string()],
        ..PatternsConfig::default()
    };

    let catalog = build_catalog(&records, &config);

    // Excluded file must not contribute to the catalog.
    if let Some(cat) = catalog.entries.get("error_handling") {
        for stats in cat.values() {
            for loc in &stats.locations {
                assert_ne!(
                    loc.file,
                    PathBuf::from("src/vendor/generated.rs"),
                    "excluded file must not appear in catalog locations"
                );
            }
        }
    }

    // The FeatureRecord slice still contains the excluded file — the catalog
    // builder does not mutate the input slice.
    let excluded_record = records
        .iter()
        .find(|r| r.path == PathBuf::from("src/vendor/generated.rs"));
    assert!(
        excluded_record.is_some(),
        "excluded file must remain in the FeatureRecord slice"
    );
}

/// A non-excluded file IS present in the catalog.
#[test]
fn non_excluded_file_present_in_catalog() {
    let hints = vec![hint("try_expression"), hint("try_expression")];
    let records = vec![record("src/lib.rs", hints)];

    let config = PatternsConfig {
        min_pattern_nodes: 1,
        scope_exclude: vec!["src/vendor/**".to_string()],
        ..PatternsConfig::default()
    };

    let catalog = build_catalog(&records, &config);
    assert!(
        catalog.entries.contains_key("error_handling"),
        "non-excluded file must appear in catalog"
    );
}

/// Empty `scope_exclude` applies no filtering.
#[test]
fn empty_scope_exclude_applies_no_filtering() {
    let hints = vec![hint("try_expression")];
    let records = vec![record("src/vendor/foo.rs", hints)];

    let config = PatternsConfig {
        min_pattern_nodes: 1,
        scope_exclude: vec![],
        ..PatternsConfig::default()
    };

    let catalog = build_catalog(&records, &config);
    assert!(
        catalog.entries.contains_key("error_handling"),
        "with empty scope_exclude no files should be filtered"
    );
}

/// Multiple `scope_exclude` globs exclude all matching files.
#[test]
fn multiple_scope_exclude_patterns() {
    let vendor = record("vendor/a.rs", vec![hint("try_expression")]);
    let gen = record("generated/b.rs", vec![hint("match_expression")]);
    let normal = record("src/c.rs", vec![hint("await_expression")]);

    let records = vec![vendor, gen, normal];

    let config = PatternsConfig {
        min_pattern_nodes: 1,
        scope_exclude: vec!["vendor/**".to_string(), "generated/**".to_string()],
        ..PatternsConfig::default()
    };

    let catalog = build_catalog(&records, &config);

    // Only async_patterns from src/c.rs should appear.
    assert!(
        catalog.entries.contains_key("async_patterns"),
        "non-excluded file must appear"
    );
    // error_handling patterns from excluded files must be absent.
    assert!(
        !catalog.entries.contains_key("error_handling"),
        "patterns from excluded files must not appear"
    );
}
