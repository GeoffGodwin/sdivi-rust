//! M45.2 integration tests: Python/Java error_handling detection via `build_catalog`.
//!
//! Verifies that `except_clause`, `catch_clause`, and `throw_statement` nodes —
//! already collected by their respective adapters but previously dropped —
//! are now routed to the `error_handling` catalog bucket.
//!
//! Key double-count semantic: a Python `try` with 3 `except` arms yields
//! 1 `try_statement` + 3 `except_clause` = 4 `error_handling` instances.
//! This is intentional — more arms = more structure = higher entropy.
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

/// M45.2: Python `except_clause` routes to error_handling.
#[test]
fn python_except_clause_routes_to_error_handling() {
    let records = vec![make_record(
        "handler.py",
        "python",
        vec![make_hint("except_clause", "except ValueError as e:", 5)],
    )];
    let catalog = build_catalog(&records, &min1_config());
    assert!(
        catalog.entries.contains_key("error_handling"),
        "build_catalog must produce an `error_handling` bucket for except_clause (M45.2). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    let total: u32 = catalog.entries["error_handling"]
        .values()
        .map(|s| s.count)
        .sum();
    assert_eq!(
        total, 1,
        "error_handling bucket must contain 1 except_clause"
    );
}

/// M45.2: multi-arm Python try/except — each except_clause counted separately.
///
/// A `try` with 3 `except` arms emits 1 `try_statement` + 3 `except_clause` = 4 instances.
/// This is intentional: more arms = more error-flow structure = higher entropy signal.
#[test]
fn python_multi_arm_except_counts_each_clause() {
    let records = vec![make_record(
        "handler.py",
        "python",
        vec![
            make_hint("try_statement", "try:", 1),
            make_hint("except_clause", "except ValueError:", 3),
            make_hint("except_clause", "except (TypeError, KeyError) as e:", 5),
            make_hint("except_clause", "except Exception:", 7),
        ],
    )];
    let catalog = build_catalog(&records, &min1_config());
    assert!(
        catalog.entries.contains_key("error_handling"),
        "error_handling bucket must be present for multi-arm try/except"
    );
    let total: u32 = catalog.entries["error_handling"]
        .values()
        .map(|s| s.count)
        .sum();
    assert_eq!(
        total, 4,
        "1 try_statement + 3 except_clause = 4 error_handling instances (M45.2 double-count semantic)"
    );
}

/// M45.2: Java `catch_clause` routes to error_handling.
#[test]
fn java_catch_clause_routes_to_error_handling() {
    let records = vec![make_record(
        "Service.java",
        "java",
        vec![make_hint("catch_clause", "catch (IOException e) { }", 10)],
    )];
    let catalog = build_catalog(&records, &min1_config());
    assert!(
        catalog.entries.contains_key("error_handling"),
        "build_catalog must produce an `error_handling` bucket for catch_clause (M45.2). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    let total: u32 = catalog.entries["error_handling"]
        .values()
        .map(|s| s.count)
        .sum();
    assert_eq!(
        total, 1,
        "error_handling bucket must contain 1 catch_clause"
    );
}

/// M45.2: Java `throw_statement` routes to error_handling.
#[test]
fn java_throw_statement_routes_to_error_handling() {
    let records = vec![make_record(
        "Service.java",
        "java",
        vec![make_hint(
            "throw_statement",
            "throw new RuntimeException(msg)",
            20,
        )],
    )];
    let catalog = build_catalog(&records, &min1_config());
    assert!(
        catalog.entries.contains_key("error_handling"),
        "build_catalog must produce an `error_handling` bucket for throw_statement (M45.2). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    let total: u32 = catalog.entries["error_handling"]
        .values()
        .map(|s| s.count)
        .sum();
    assert_eq!(
        total, 1,
        "error_handling bucket must contain 1 throw_statement"
    );
}

/// Mixed Java error-handling: try, catch, and throw all count.
#[test]
fn java_mixed_error_handling_counts() {
    let records = vec![make_record(
        "Service.java",
        "java",
        vec![
            make_hint("catch_clause", "catch (IOException e) { }", 5),
            make_hint("catch_clause", "catch (SQLException e) { }", 10),
            make_hint("throw_statement", "throw new RuntimeException(e)", 15),
        ],
    )];
    let catalog = build_catalog(&records, &min1_config());
    assert!(
        catalog.entries.contains_key("error_handling"),
        "error_handling bucket must be present for mixed Java catch/throw"
    );
    let total: u32 = catalog.entries["error_handling"]
        .values()
        .map(|s| s.count)
        .sum();
    assert_eq!(
        total, 3,
        "2 catch_clause + 1 throw_statement = 3 error_handling instances"
    );
}
