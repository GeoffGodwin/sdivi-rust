//! M45.1 acceptance-criterion tests: `resource_management` enriched with Python/Go/Java.
//!
//! Verifies that:
//! - `category_for_node_kind("with_statement", "python") == Some("resource_management")`
//! - `category_for_node_kind("defer_statement", "go") == Some("resource_management")`
//! - `category_for_node_kind("try_with_resources_statement", "java") == Some("resource_management")`
//! - `classify_hint` routes all three node kinds via the `other` fall-through arm.
//! - Rust `macro_invocation` behaviour is unchanged.

use sdivi_patterns::queries::{category_for_node_kind, classify_hint};
use sdivi_patterns::PatternHintInput;

fn hint(node_kind: &str, text: &str) -> PatternHintInput {
    PatternHintInput {
        node_kind: node_kind.to_string(),
        text: text.to_string(),
    }
}

// ── M45.1 acceptance criteria ─────────────────────────────────────────────────

#[test]
fn with_statement_is_resource_management() {
    assert_eq!(
        category_for_node_kind("with_statement", "python"),
        Some("resource_management"),
        "with_statement must map to resource_management (M45.1 acceptance criterion)"
    );
}

#[test]
fn defer_statement_is_resource_management() {
    assert_eq!(
        category_for_node_kind("defer_statement", "go"),
        Some("resource_management"),
        "defer_statement must map to resource_management (M45.1 acceptance criterion)"
    );
}

#[test]
fn try_with_resources_statement_is_resource_management() {
    assert_eq!(
        category_for_node_kind("try_with_resources_statement", "java"),
        Some("resource_management"),
        "try_with_resources_statement must map to resource_management (M45.1 acceptance criterion)"
    );
}

// ── classify_hint routing — new node kinds ────────────────────────────────────

#[test]
fn classify_hint_with_statement_routes_to_resource_management() {
    // with_statement is not call_expression or macro_invocation — falls through to
    // category_for_node_kind via the `other` arm.
    let result = classify_hint(&hint("with_statement", "with open(p) as f:"), "python");
    assert_eq!(result, vec!["resource_management"]);
}

#[test]
fn classify_hint_defer_statement_routes_to_resource_management() {
    let result = classify_hint(&hint("defer_statement", "defer f.Close()"), "go");
    assert_eq!(result, vec!["resource_management"]);
}

#[test]
fn classify_hint_try_with_resources_routes_to_resource_management() {
    let result = classify_hint(
        &hint("try_with_resources_statement", "try (var r = open(p)) { }"),
        "java",
    );
    assert_eq!(result, vec!["resource_management"]);
}

// ── Rust macro_invocation unchanged ───────────────────────────────────────────

#[test]
fn macro_invocation_non_logging_stays_resource_management() {
    let result = classify_hint(&hint("macro_invocation", "vec![1, 2, 3]"), "rust");
    assert_eq!(result, vec!["resource_management"]);
}

#[test]
fn macro_invocation_logging_still_routes_to_logging() {
    let result = classify_hint(&hint("macro_invocation", "tracing::info!(\"hi\")"), "rust");
    assert_eq!(result, vec!["logging"]);
}

// ── Boundary: defer_statement not concurrency (M44 boundary holds) ───────────

#[test]
fn defer_statement_is_not_concurrency() {
    let result = classify_hint(&hint("defer_statement", "defer mu.Unlock()"), "go");
    assert!(
        !result.contains(&"concurrency"),
        "defer_statement must not map to concurrency after M45.1; got {result:?}"
    );
    assert_eq!(
        result,
        vec!["resource_management"],
        "defer_statement must map to resource_management (not concurrency) after M45.1"
    );
}

// ── Category count unchanged ──────────────────────────────────────────────────

#[test]
fn list_categories_count_after_m45_1() {
    // M45.1 adds node kinds within an existing category; M46 adds comprehensions (+1).
    let catalog = sdivi_core::list_categories();
    assert_eq!(
        catalog.categories.len(),
        19,
        "list_categories must return exactly 19 categories (18 at M45.1 + comprehensions at M46)"
    );
}

#[test]
fn list_categories_includes_resource_management() {
    let catalog = sdivi_core::list_categories();
    let names: Vec<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.contains(&"resource_management"),
        "list_categories must include 'resource_management'"
    );
}
