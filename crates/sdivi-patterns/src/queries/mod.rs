//! Per-category node-kind classification rules.
//!
//! Each sub-module declares the tree-sitter node kinds that map to a built-in
//! pattern category. Two classifiers are provided:
//!
//! - [`category_for_node_kind`] — node-kind-only, fast, does not inspect source text.
//! - [`classify_hint`] — node-kind + callee-text, uses per-language regex tables.
//!   Prefer this for foreign extractors that have full [`PatternHintInput`] access.
//!
//! Category names are stable from Milestone 6 forward. Renaming any name is a
//! breaking change requiring a `MIGRATION_NOTES.md` entry.

pub mod async_patterns;
pub mod class_hierarchy;
pub mod data_access;
pub mod decorators;
pub mod error_handling;
pub mod framework_hooks;
pub mod logging;
pub mod resource_management;
pub mod state_management;
pub mod type_assertions;

use crate::hint_input::PatternHintInput;

/// All built-in category names in stable alphabetical order.
///
/// Note: `logging` is classified via [`classify_hint`] callee-text inspection only;
/// [`category_for_node_kind`] never returns `Some("logging")`.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::ALL_CATEGORIES;
///
/// assert!(ALL_CATEGORIES.contains(&"decorators"));
/// assert!(ALL_CATEGORIES.contains(&"logging"));
/// assert_eq!(ALL_CATEGORIES.len(), 10);
/// ```
pub const ALL_CATEGORIES: &[&str] = &[
    "async_patterns",
    "class_hierarchy",
    "data_access",
    "decorators",
    "error_handling",
    "framework_hooks",
    "logging",
    "resource_management",
    "state_management",
    "type_assertions",
];

/// Maps a tree-sitter `node_kind` to the built-in category it belongs to.
///
/// Returns `None` if the node kind does not belong to any category.
/// The `_language` parameter is reserved for future per-language overrides.
///
/// **Used internally by `classify_hint` for non-call/non-macro node kinds;
/// the native pipeline no longer calls this function directly (as of M33).**
/// Foreign extractors with full [`PatternHintInput`] access should call
/// [`classify_hint`] instead. This function is preserved for callers that have a
/// node kind but no source text — and for backward compatibility with embedders
/// that integrated against the M29 API.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::category_for_node_kind;
///
/// assert_eq!(category_for_node_kind("try_expression", "rust"), Some("error_handling"));
/// assert_eq!(category_for_node_kind("await_expression", "rust"), Some("async_patterns"));
/// assert_eq!(category_for_node_kind("unknown_node", "rust"), None);
/// ```
///
/// # See also
///
/// [`classify_hint`] — callee-aware classifier; preferred for most callers (M33+).
pub fn category_for_node_kind(node_kind: &str, _language: &str) -> Option<&'static str> {
    if async_patterns::NODE_KINDS.contains(&node_kind) {
        Some("async_patterns")
    } else if class_hierarchy::NODE_KINDS.contains(&node_kind) {
        Some("class_hierarchy")
    } else if data_access::NODE_KINDS.contains(&node_kind) {
        Some("data_access")
    } else if decorators::NODE_KINDS.contains(&node_kind) {
        Some("decorators")
    } else if error_handling::NODE_KINDS.contains(&node_kind) {
        Some("error_handling")
    } else if resource_management::NODE_KINDS.contains(&node_kind) {
        Some("resource_management")
    } else if state_management::NODE_KINDS.contains(&node_kind) {
        Some("state_management")
    } else if type_assertions::NODE_KINDS.contains(&node_kind) {
        Some("type_assertions")
    } else {
        None
    }
}

#[allow(clippy::type_complexity)] // P1 > P6=framework_hooks > P8=logging > P9=data_access; future milestones insert at their slot
const CALL_DISPATCH: &[(&str, fn(&str, &str) -> bool)] = &[
    ("async_patterns", async_patterns::matches_callee),
    ("framework_hooks", framework_hooks::matches_callee),
    ("logging", logging::matches_callee),
    ("data_access", data_access::matches_callee),
];
/// Classify a [`PatternHintInput`] using both node kind and callee-text inspection.
///
/// Returns a `Vec` of category name(s) the hint belongs to. In v0 the return is
/// always 0 or 1 entries — the regex tables are designed to be disjoint per language.
/// The `Vec` return is forward-looking: a future category that legitimately co-occurs
/// with another (e.g. `console.error(err)` as both `logging` and `error_handling`)
/// can be added without an API break.
///
/// ## Dispatch order for `call_expression` / `call`
///
/// Iterates [`CALL_DISPATCH`] in order; first match wins (P1/P6/P8/P9 active at M35).
///
/// ## `macro_invocation`
///
/// Defaults to `resource_management`. Rust logging macros (`tracing::*!`,
/// `log::*!`, `println!`, `eprintln!`, `print!`, `eprint!`, `dbg!`) are
/// reclassified as `logging` via [`resource_management::excludes_callee`].
///
/// ## Other node kinds
///
/// Falls through to [`category_for_node_kind`] — the existing node-kind-only table.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::classify_hint;
/// use sdivi_patterns::PatternHintInput;
///
/// let hint = PatternHintInput {
///     node_kind: "call_expression".to_string(),
///     text: "console.log(\"x\")".to_string(),
/// };
/// assert_eq!(classify_hint(&hint, "typescript"), vec!["logging"]);
///
/// let mac = PatternHintInput {
///     node_kind: "macro_invocation".to_string(),
///     text: "vec![1, 2, 3]".to_string(),
/// };
/// assert_eq!(classify_hint(&mac, "rust"), vec!["resource_management"]);
/// ```
pub fn classify_hint(hint: &PatternHintInput, language: &str) -> Vec<&'static str> {
    match hint.node_kind.as_str() {
        "call_expression" | "call" => {
            for &(category, matches) in CALL_DISPATCH {
                if matches(&hint.text, language) {
                    return vec![category];
                }
            }
            vec![]
        }
        "macro_invocation" => {
            if resource_management::excludes_callee(&hint.text, language)
                && logging::matches_callee(&hint.text, language)
            {
                return vec!["logging"];
            }
            vec!["resource_management"]
        }
        other => category_for_node_kind(other, language)
            .map(|c| vec![c])
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_expression_is_error_handling() {
        assert_eq!(
            category_for_node_kind("try_expression", "rust"),
            Some("error_handling")
        );
    }

    #[test]
    fn await_expression_is_async_patterns() {
        assert_eq!(
            category_for_node_kind("await_expression", "rust"),
            Some("async_patterns")
        );
    }

    #[test]
    fn closure_expression_is_state_management() {
        assert_eq!(
            category_for_node_kind("closure_expression", "rust"),
            Some("state_management")
        );
    }

    #[test]
    fn macro_invocation_is_resource_management() {
        assert_eq!(
            category_for_node_kind("macro_invocation", "rust"),
            Some("resource_management")
        );
    }

    #[test]
    fn unknown_node_kind_returns_none() {
        assert_eq!(category_for_node_kind("unknown_xyz", "rust"), None);
    }

    #[test]
    fn all_categories_has_ten_entries() {
        assert_eq!(ALL_CATEGORIES.len(), 10);
        assert!(ALL_CATEGORIES.contains(&"framework_hooks"));
        assert!(ALL_CATEGORIES.contains(&"decorators"));
    }

    #[test]
    fn logging_is_in_all_categories() {
        assert!(ALL_CATEGORIES.contains(&"logging"));
    }

    #[test]
    fn class_hierarchy_is_in_all_categories() {
        assert!(ALL_CATEGORIES.contains(&"class_hierarchy"));
    }

    #[test]
    fn class_declaration_is_class_hierarchy() {
        assert_eq!(
            category_for_node_kind("class_declaration", "typescript"),
            Some("class_hierarchy")
        );
    }

    #[test]
    fn class_definition_is_class_hierarchy() {
        assert_eq!(
            category_for_node_kind("class_definition", "python"),
            Some("class_hierarchy")
        );
    }

    #[test]
    fn impl_item_is_class_hierarchy() {
        assert_eq!(
            category_for_node_kind("impl_item", "rust"),
            Some("class_hierarchy")
        );
    }

    #[test]
    fn interface_declaration_is_class_hierarchy() {
        assert_eq!(
            category_for_node_kind("interface_declaration", "java"),
            Some("class_hierarchy")
        );
    }

    #[test]
    fn abstract_class_declaration_is_class_hierarchy() {
        assert_eq!(
            category_for_node_kind("abstract_class_declaration", "typescript"),
            Some("class_hierarchy")
        );
    }

    // M30 sentinel: tests `category_for_node_kind` (node-kind-only, unchanged).
    // M33 promoted `logging` via `classify_hint`; `category_for_node_kind` is
    // intentionally unchanged. See `tests/m33_sentinels.rs` for the M33 counterpart.
    #[test]
    fn category_for_node_kind_never_returns_logging() {
        // `category_for_node_kind` never returns logging — that requires callee-text
        // inspection (see `classify_hint`). This is unchanged through M32 and M33.
        for kind in ["call_expression", "call", "macro_invocation"] {
            for lang in ["rust", "python", "typescript", "javascript", "go", "java"] {
                assert_ne!(
                    category_for_node_kind(kind, lang),
                    Some("logging"),
                    "logging is catalog-only in v0 for category_for_node_kind; \
                     routing for ({kind}, {lang}) would steal from data_access/resource_management"
                );
            }
        }
    }

    #[test]
    fn call_expression_is_data_access() {
        assert_eq!(category_for_node_kind("call_expression", "typescript"), Some("data_access"));
        assert_eq!(category_for_node_kind("call", "python"), Some("data_access"));
    }

    #[test]
    fn decorator_is_decorators() {
        assert_eq!(category_for_node_kind("decorator", "typescript"), Some("decorators"));
        assert_eq!(category_for_node_kind("decorator", "javascript"), Some("decorators"));
    }
}
