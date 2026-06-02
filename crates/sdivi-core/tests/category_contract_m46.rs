//! M46 acceptance-criterion tests: `comprehensions` pattern category.
//!
//! Verifies that:
//! - `category_for_node_kind("list_comprehension", "python") == Some("comprehensions")`
//! - `category_for_node_kind("set_comprehension", "python") == Some("comprehensions")`
//! - `category_for_node_kind("dictionary_comprehension", "python") == Some("comprehensions")`
//! - `category_for_node_kind("generator_expression", "python") == Some("comprehensions")`
//! - Non-Python languages return `None` for these node kinds (no match in other grammars).
//! - `classify_hint` routes all four node kinds via the `other` fall-through arm.
//! - `list_categories()` count is now 19 (added `comprehensions`).

use sdivi_patterns::queries::{category_for_node_kind, classify_hint};
use sdivi_patterns::PatternHintInput;

fn hint(node_kind: &str, text: &str) -> PatternHintInput {
    PatternHintInput {
        node_kind: node_kind.to_string(),
        text: text.to_string(),
    }
}

// ── M46 acceptance criteria ───────────────────────────────────────────────────

#[test]
fn list_comprehension_is_comprehensions() {
    assert_eq!(
        category_for_node_kind("list_comprehension", "python"),
        Some("comprehensions"),
        "list_comprehension must map to comprehensions (M46 acceptance criterion)"
    );
}

#[test]
fn set_comprehension_is_comprehensions() {
    assert_eq!(
        category_for_node_kind("set_comprehension", "python"),
        Some("comprehensions"),
        "set_comprehension must map to comprehensions (M46 acceptance criterion)"
    );
}

#[test]
fn dictionary_comprehension_is_comprehensions() {
    assert_eq!(
        category_for_node_kind("dictionary_comprehension", "python"),
        Some("comprehensions"),
        "dictionary_comprehension must map to comprehensions (M46 acceptance criterion)"
    );
}

#[test]
fn generator_expression_is_comprehensions() {
    assert_eq!(
        category_for_node_kind("generator_expression", "python"),
        Some("comprehensions"),
        "generator_expression must map to comprehensions (M46 acceptance criterion)"
    );
}

// ── Non-match for other node kinds ───────────────────────────────────────────

#[test]
fn rust_await_expression_is_not_comprehensions() {
    assert_ne!(
        category_for_node_kind("await_expression", "rust"),
        Some("comprehensions"),
    );
}

#[test]
fn unknown_node_kind_is_not_comprehensions() {
    assert_eq!(category_for_node_kind("unknown_node", "python"), None);
}

// ── classify_hint routing ─────────────────────────────────────────────────────

#[test]
fn classify_hint_list_comprehension_routes_to_comprehensions() {
    let result = classify_hint(&hint("list_comprehension", "[x for x in xs]"), "python");
    assert_eq!(result, vec!["comprehensions"]);
}

#[test]
fn classify_hint_generator_expression_routes_to_comprehensions() {
    let result = classify_hint(&hint("generator_expression", "(x for x in xs)"), "python");
    assert_eq!(result, vec!["comprehensions"]);
}

#[test]
fn classify_hint_dictionary_comprehension_routes_to_comprehensions() {
    let result = classify_hint(
        &hint("dictionary_comprehension", "{k: v for k, v in items}"),
        "python",
    );
    assert_eq!(result, vec!["comprehensions"]);
}

#[test]
fn classify_hint_set_comprehension_routes_to_comprehensions() {
    let result = classify_hint(&hint("set_comprehension", "{x for x in xs}"), "python");
    assert_eq!(result, vec!["comprehensions"]);
}

// ── Category count: 19 ────────────────────────────────────────────────────────

#[test]
fn list_categories_count_is_nineteen() {
    let catalog = sdivi_core::list_categories();
    assert_eq!(
        catalog.categories.len(),
        19,
        "list_categories must return exactly 19 categories after M46 added comprehensions"
    );
}

#[test]
fn list_categories_includes_comprehensions() {
    let catalog = sdivi_core::list_categories();
    let names: Vec<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.contains(&"comprehensions"),
        "list_categories must include the 'comprehensions' category (M46)"
    );
}
