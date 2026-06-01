//! M45.2 acceptance-criterion tests: `error_handling` enriched with Python/Java clause kinds.
//!
//! Verifies that:
//! - `category_for_node_kind("except_clause", "python") == Some("error_handling")`
//! - `category_for_node_kind("catch_clause", "java") == Some("error_handling")`
//! - `category_for_node_kind("throw_statement", "java") == Some("error_handling")`
//! - `classify_hint` routes all three node kinds via the `other` fall-through arm.
//! - Rust `try_expression`/`match_expression` behaviour is unchanged.
//! - `list_categories()` count stays 18 (additive node kinds only).

use sdivi_patterns::queries::{category_for_node_kind, classify_hint};
use sdivi_patterns::PatternHintInput;

fn hint(node_kind: &str, text: &str) -> PatternHintInput {
    PatternHintInput {
        node_kind: node_kind.to_string(),
        text: text.to_string(),
    }
}

// ── M45.2 acceptance criteria ─────────────────────────────────────────────────

#[test]
fn except_clause_is_error_handling() {
    assert_eq!(
        category_for_node_kind("except_clause", "python"),
        Some("error_handling"),
        "except_clause must map to error_handling (M45.2 acceptance criterion)"
    );
}

#[test]
fn catch_clause_is_error_handling() {
    assert_eq!(
        category_for_node_kind("catch_clause", "java"),
        Some("error_handling"),
        "catch_clause must map to error_handling (M45.2 acceptance criterion)"
    );
}

#[test]
fn throw_statement_is_error_handling() {
    assert_eq!(
        category_for_node_kind("throw_statement", "java"),
        Some("error_handling"),
        "throw_statement must map to error_handling (M45.2 acceptance criterion)"
    );
}

// ── classify_hint routing — new node kinds ────────────────────────────────────

#[test]
fn classify_hint_except_clause_routes_to_error_handling() {
    let result = classify_hint(&hint("except_clause", "except ValueError as e:"), "python");
    assert_eq!(result, vec!["error_handling"]);
}

#[test]
fn classify_hint_catch_clause_routes_to_error_handling() {
    let result = classify_hint(&hint("catch_clause", "catch (IOException e) { }"), "java");
    assert_eq!(result, vec!["error_handling"]);
}

#[test]
fn classify_hint_throw_statement_routes_to_error_handling() {
    let result = classify_hint(&hint("throw_statement", "throw new RuntimeException(msg)"), "java");
    assert_eq!(result, vec!["error_handling"]);
}

// ── Rust try_expression / match_expression unchanged ─────────────────────────

#[test]
fn try_expression_still_error_handling() {
    assert_eq!(
        category_for_node_kind("try_expression", "rust"),
        Some("error_handling"),
        "try_expression must still map to error_handling after M45.2"
    );
}

#[test]
fn match_expression_still_error_handling() {
    assert_eq!(
        category_for_node_kind("match_expression", "rust"),
        Some("error_handling"),
        "match_expression must still map to error_handling after M45.2"
    );
}

// ── Category count unchanged ──────────────────────────────────────────────────

#[test]
fn list_categories_count_after_m45_2() {
    // M45.2 adds node kinds within existing category; M46 adds comprehensions (+1).
    let catalog = sdivi_core::list_categories();
    assert_eq!(
        catalog.categories.len(),
        19,
        "list_categories must return exactly 19 categories (18 at M45.2 + comprehensions at M46)"
    );
}

#[test]
fn list_categories_includes_error_handling() {
    let catalog = sdivi_core::list_categories();
    let names: Vec<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.contains(&"error_handling"),
        "list_categories must include 'error_handling'"
    );
}
