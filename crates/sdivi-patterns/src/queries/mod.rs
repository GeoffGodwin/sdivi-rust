//! Per-category node-kind classification rules.
//!
//! Each sub-module declares the tree-sitter node kinds that map to a built-in
//! pattern category. Classification is performed by [`category_for_node_kind`],
//! which returns the category name or `None` if the node kind is unrecognised.
//!
//! Category names are stable from Milestone 6 forward. Renaming any name is a
//! breaking change requiring a `MIGRATION_NOTES.md` entry.

pub mod async_patterns;
pub mod class_hierarchy;
pub mod data_access;
pub mod error_handling;
pub mod logging;
pub mod resource_management;
pub mod state_management;
pub mod type_assertions;

/// All built-in category names in stable alphabetical order.
///
/// Note: `logging` is a catalog-only category for `snapshot_version "1.0"`.
/// It is present here so embedders can emit `PatternInstanceInput { category: "logging", … }`
/// and have those instances round-trip through `compute_pattern_metrics` and
/// `compute_delta`. [`category_for_node_kind`] never returns `Some("logging")` —
/// the relevant node kinds overlap with `data_access` and `resource_management`.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::ALL_CATEGORIES;
///
/// assert!(ALL_CATEGORIES.contains(&"error_handling"));
/// assert!(ALL_CATEGORIES.contains(&"async_patterns"));
/// assert!(ALL_CATEGORIES.contains(&"data_access"));
/// assert!(ALL_CATEGORIES.contains(&"logging"));
/// assert_eq!(ALL_CATEGORIES.len(), 8);
/// ```
pub const ALL_CATEGORIES: &[&str] = &[
    "async_patterns",
    "class_hierarchy",
    "data_access",
    "error_handling",
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
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::category_for_node_kind;
///
/// assert_eq!(category_for_node_kind("try_expression", "rust"), Some("error_handling"));
/// assert_eq!(category_for_node_kind("await_expression", "rust"), Some("async_patterns"));
/// assert_eq!(category_for_node_kind("unknown_node", "rust"), None);
/// ```
pub fn category_for_node_kind(node_kind: &str, _language: &str) -> Option<&'static str> {
    if async_patterns::NODE_KINDS.contains(&node_kind) {
        Some("async_patterns")
    } else if class_hierarchy::NODE_KINDS.contains(&node_kind) {
        Some("class_hierarchy")
    } else if data_access::NODE_KINDS.contains(&node_kind) {
        Some("data_access")
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
    fn all_categories_has_eight_entries() {
        assert_eq!(ALL_CATEGORIES.len(), 8);
        assert!(ALL_CATEGORIES.contains(&"data_access"));
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

    #[test]
    fn category_for_node_kind_never_returns_logging() {
        // logging is a catalog-only category for v0 — foreign extractors emit
        // it directly. Document the contract here so a future change that adds
        // a native routing must update this test deliberately, not by accident.
        for kind in ["call_expression", "call", "macro_invocation"] {
            for lang in ["rust", "python", "typescript", "javascript", "go", "java"] {
                assert_ne!(
                    category_for_node_kind(kind, lang),
                    Some("logging"),
                    "logging is catalog-only in v0; routing for ({kind}, {lang}) \
                     would steal from data_access/resource_management"
                );
            }
        }
    }

    #[test]
    fn call_expression_is_data_access() {
        assert_eq!(
            category_for_node_kind("call_expression", "typescript"),
            Some("data_access")
        );
        assert_eq!(
            category_for_node_kind("call", "python"),
            Some("data_access")
        );
    }
}
