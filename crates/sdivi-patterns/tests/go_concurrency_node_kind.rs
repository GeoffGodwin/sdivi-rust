//! Tests for Go concurrency node-kind classification.
//!
//! DRIFT_LOG entries for `category_for_node_kind("go_statement", "go")` and related
//! Go concurrency node kinds ensure they are properly classified.

use sdivi_patterns::queries::category_for_node_kind;
use sdivi_patterns::queries::concurrency;
use sdivi_patterns::queries::ALL_CATEGORIES;

/// Verifies that `category_for_node_kind("go_statement", "go")` returns `Some("concurrency")`.
///
/// This test addresses a DRIFT_LOG note that the node-kind path for Go concurrency
/// kinds (`go_statement` and `select_statement`) has no direct unit-test coverage
/// and is covered only via integration tests in the Go adapter.
#[test]
fn go_statement_maps_to_concurrency_category() {
    let result = category_for_node_kind("go_statement", "go");
    assert_eq!(
        result,
        Some("concurrency"),
        "go_statement must map to concurrency category"
    );
}

/// Verifies that `select_statement` (another Go concurrency node kind) maps correctly.
#[test]
fn select_statement_maps_to_concurrency_category() {
    let result = category_for_node_kind("select_statement", "go");
    assert_eq!(
        result,
        Some("concurrency"),
        "select_statement must map to concurrency category"
    );
}

/// Verifies that Go concurrency node kinds work regardless of language parameter.
///
/// The `_language` parameter is reserved for future per-language overrides, but
/// currently both Go and non-Go languages should return the same result for
/// these Go-specific node kinds (since `category_for_node_kind` ignores language).
#[test]
fn go_statement_language_parameter_ignored() {
    assert_eq!(
        category_for_node_kind("go_statement", "python"),
        Some("concurrency")
    );
    assert_eq!(
        category_for_node_kind("go_statement", "rust"),
        Some("concurrency")
    );
    assert_eq!(
        category_for_node_kind("go_statement", "typescript"),
        Some("concurrency")
    );
    assert_eq!(
        category_for_node_kind("select_statement", "python"),
        Some("concurrency")
    );
    assert_eq!(
        category_for_node_kind("select_statement", "rust"),
        Some("concurrency")
    );
    assert_eq!(
        category_for_node_kind("select_statement", "typescript"),
        Some("concurrency")
    );
}

/// Verifies that invalid or unknown Go node kinds return `None`.
#[test]
fn unknown_go_node_kinds_return_none() {
    assert_eq!(category_for_node_kind("go_foo_statement", "go"), None);
    assert_eq!(category_for_node_kind("unknown_node", "go"), None);
}

/// Verifies that `defer_statement` maps to `resource_management`, not `concurrency`.
///
/// `defer_statement` is a known Go node kind that belongs to `resource_management`
/// (M45.1). This test guards against it being accidentally pulled into `concurrency`.
#[test]
fn defer_statement_maps_to_resource_management() {
    assert_eq!(
        category_for_node_kind("defer_statement", "go"),
        Some("resource_management")
    );
}

/// Verifies that all `concurrency::NODE_KINDS` entries are classified correctly.
///
/// Iterates directly over `concurrency::NODE_KINDS` so this test stays in sync
/// automatically if the constant grows — no manual list to maintain.
#[test]
fn all_concurrency_node_kinds_are_classified() {
    for node_kind in concurrency::NODE_KINDS {
        let result = category_for_node_kind(node_kind, "go");
        assert_eq!(
            result,
            Some("concurrency"),
            "{} should be classified as concurrency",
            node_kind
        );
    }
}

/// Verifies that Go concurrency node kinds are not misclassified as other categories.
///
/// Iterates over `ALL_CATEGORIES` so this test stays in sync automatically when new
/// categories are added — no manual list to maintain.
#[test]
fn go_statement_not_misclassified() {
    for cat in ALL_CATEGORIES {
        if *cat == "concurrency" {
            assert_eq!(
                category_for_node_kind("go_statement", "go"),
                Some("concurrency"),
                "go_statement must map to concurrency"
            );
        } else {
            assert_ne!(
                category_for_node_kind("go_statement", "go"),
                Some(*cat),
                "go_statement must not map to {}",
                cat
            );
        }
    }
}
