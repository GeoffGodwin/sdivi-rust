//! Node kinds classified as comprehension usage.
//!
//! Python comprehension forms — list comprehensions, set comprehensions,
//! dictionary comprehensions, and generator expressions — are classified here.
//! These constructs are Python-specific; no equivalent dedicated node kind exists
//! in the other supported grammars. Cross-language consumers see an empty bucket
//! for non-Python repos, consistent with how `class_hierarchy` reads empty for Go.
//!
//! ## Count semantics
//!
//! One instance per comprehension node. Nested comprehensions emit one node each —
//! `[x for row in matrix for x in row]` is one `list_comprehension` node (one instance),
//! while `[[x for x in row] for row in matrix]` contains an inner `list_comprehension`
//! nested inside an outer one and emits two instances.

/// Tree-sitter node kinds for comprehension usage.
///
/// All four forms are Python-only in v0:
///
/// - `dictionary_comprehension` — `{k: v for k, v in items}`
/// - `generator_expression` — `(x for x in xs)`, `sum(x*x for x in xs)`
/// - `list_comprehension` — `[x for x in xs]`
/// - `set_comprehension` — `{x for x in xs}`
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::comprehensions::NODE_KINDS;
///
/// assert!(NODE_KINDS.contains(&"list_comprehension"));
/// assert!(NODE_KINDS.contains(&"set_comprehension"));
/// assert!(NODE_KINDS.contains(&"dictionary_comprehension"));
/// assert!(NODE_KINDS.contains(&"generator_expression"));
/// assert_eq!(NODE_KINDS.len(), 4);
/// ```
pub const NODE_KINDS: &[&str] = &[
    "dictionary_comprehension",
    "generator_expression",
    "list_comprehension",
    "set_comprehension",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_kinds_contains_list_comprehension() {
        assert!(NODE_KINDS.contains(&"list_comprehension"));
    }

    #[test]
    fn node_kinds_contains_set_comprehension() {
        assert!(NODE_KINDS.contains(&"set_comprehension"));
    }

    #[test]
    fn node_kinds_contains_dictionary_comprehension() {
        assert!(NODE_KINDS.contains(&"dictionary_comprehension"));
    }

    #[test]
    fn node_kinds_contains_generator_expression() {
        assert!(NODE_KINDS.contains(&"generator_expression"));
    }

    #[test]
    fn node_kinds_has_four_entries() {
        assert_eq!(NODE_KINDS.len(), 4);
    }

    #[test]
    fn non_comprehension_node_kinds_do_not_match() {
        assert!(!NODE_KINDS.contains(&"await_expression"));
        assert!(!NODE_KINDS.contains(&"closure_expression"));
    }
}
