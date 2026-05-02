//! Per-category node-kind classification rules.
//!
//! Each sub-module declares the tree-sitter node kinds that map to a built-in
//! pattern category. Classification is performed by [`category_for_node_kind`],
//! which returns the category name or `None` if the node kind is unrecognised.
//!
//! Category names are stable from Milestone 6 forward. Renaming any name is a
//! breaking change requiring a `MIGRATION_NOTES.md` entry.

pub mod async_patterns;
pub mod error_handling;
pub mod resource_management;
pub mod state_management;
pub mod type_assertions;

/// All built-in category names in stable alphabetical order.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::ALL_CATEGORIES;
///
/// assert!(ALL_CATEGORIES.contains(&"error_handling"));
/// assert!(ALL_CATEGORIES.contains(&"async_patterns"));
/// assert_eq!(ALL_CATEGORIES.len(), 5);
/// ```
pub const ALL_CATEGORIES: &[&str] = &[
    "async_patterns",
    "error_handling",
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
    if error_handling::NODE_KINDS.contains(&node_kind) {
        Some("error_handling")
    } else if async_patterns::NODE_KINDS.contains(&node_kind) {
        Some("async_patterns")
    } else if state_management::NODE_KINDS.contains(&node_kind) {
        Some("state_management")
    } else if type_assertions::NODE_KINDS.contains(&node_kind) {
        Some("type_assertions")
    } else if resource_management::NODE_KINDS.contains(&node_kind) {
        Some("resource_management")
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
    fn all_categories_has_five_entries() {
        assert_eq!(ALL_CATEGORIES.len(), 5);
    }
}
