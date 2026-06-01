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
pub mod collection_pipelines;
pub mod comprehensions;
pub mod concurrency;
pub mod data_access;
pub mod decorators;
pub mod error_handling;
pub mod framework_hooks;
pub mod http_routing;
pub mod logging;
pub mod null_safety;
pub mod resource_management;
pub mod schema_validation;
pub mod serialization;
pub mod state_management;
pub mod state_store;
pub mod testing;
pub mod type_assertions;

use crate::hint_input::PatternHintInput;

/// All built-in category names in stable alphabetical order.
///
/// Note: the following categories are classified via [`classify_hint`] callee-text inspection
/// only; [`category_for_node_kind`] never returns them: `logging`, `testing`, `serialization`,
/// `schema_validation`, `state_store`, `framework_hooks`, `http_routing`, `data_access`,
/// `collection_pipelines`, and `concurrency`. Node-kind-only categories (`async_patterns`,
/// `class_hierarchy`, `comprehensions`, `decorators`, `error_handling`, `null_safety`,
/// `resource_management`, `state_management`, and `type_assertions`) are returned by both.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::ALL_CATEGORIES;
///
/// assert!(ALL_CATEGORIES.contains(&"comprehensions"));
/// assert!(ALL_CATEGORIES.contains(&"concurrency"));
/// assert!(ALL_CATEGORIES.contains(&"serialization"));
/// assert!(ALL_CATEGORIES.contains(&"testing"));
/// assert_eq!(ALL_CATEGORIES.len(), 19);
/// ```
pub const ALL_CATEGORIES: &[&str] = &[
    "async_patterns",
    "class_hierarchy",
    "collection_pipelines",
    "comprehensions",
    "concurrency",
    "data_access",
    "decorators",
    "error_handling",
    "framework_hooks",
    "http_routing",
    "logging",
    "null_safety",
    "resource_management",
    "schema_validation",
    "serialization",
    "state_management",
    "state_store",
    "testing",
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
    } else if comprehensions::NODE_KINDS.contains(&node_kind) {
        Some("comprehensions")
    } else if concurrency::NODE_KINDS.contains(&node_kind) {
        Some("concurrency")
    } else if data_access::NODE_KINDS.contains(&node_kind) {
        Some("data_access")
    } else if decorators::NODE_KINDS.contains(&node_kind) {
        Some("decorators")
    } else if error_handling::NODE_KINDS.contains(&node_kind) {
        Some("error_handling")
    } else if null_safety::NODE_KINDS.contains(&node_kind) {
        Some("null_safety")
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

#[allow(clippy::type_complexity)] // P1 > P2=testing > P3=serialization > P4=schema_validation > P5=state_store > P6=framework_hooks > P7=http_routing > P8=logging > P9=data_access > P10=collection_pipelines > P11=concurrency; future milestones insert at their slot
const CALL_DISPATCH: &[(&str, fn(&str, &str) -> bool)] = &[
    ("async_patterns", async_patterns::matches_callee),
    ("testing", testing::matches_callee),
    ("serialization", serialization::matches_callee),
    ("schema_validation", schema_validation::matches_callee),
    ("state_store", state_store::matches_callee),
    ("framework_hooks", framework_hooks::matches_callee),
    ("http_routing", http_routing::matches_callee),
    ("logging", logging::matches_callee),
    ("data_access", data_access::matches_callee),
    ("collection_pipelines", collection_pipelines::matches_callee),
    ("concurrency", concurrency::matches_callee),
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
/// Iterates [`CALL_DISPATCH`] in order; first match wins (P1/P2/P3/P4/P5/P6/P7/P8/P9/P10/P11 active at M44).
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
mod tests;
#[cfg(test)]
mod tests_m45_2;
