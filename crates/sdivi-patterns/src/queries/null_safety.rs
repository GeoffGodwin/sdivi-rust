//! Node kinds classified as null-safety usage.
//!
//! Detects TypeScript and JavaScript optional chaining (`a?.b`, `a?.()`,
//! `a?.[i]`) via the `optional_chain` node kind, and TypeScript non-null
//! assertions (`el!`) via the `non_null_expression` node kind. Detection is
//! node-kind only — no callee allowlist is applied.
//!
//! ## Language support
//!
//! - `optional_chain` — TypeScript and JavaScript (tree-sitter-typescript ≥ 0.21,
//!   tree-sitter-javascript ≥ 0.21).
//! - `non_null_expression` — TypeScript only; not present in the JavaScript
//!   tree-sitter grammar.
//!
//! ## Count semantics
//!
//! Each emitted `optional_chain` node counts as one instance. A long chain
//! `a?.b?.c` may produce nested `optional_chain` nodes — each such node counts
//! independently. This per-node counting is deterministic and reflects how
//! deeply optional-chaining is nested within an expression.
//!
//! ## Deferred: nullish coalescing (`??`)
//!
//! `a ?? b` is a `binary_expression` with a `??` operator child, not a
//! dedicated node kind. Detecting it cleanly requires operator-field inspection
//! (reading the `operator` child), which is outside the v0 node-kind model.
//! `binary_expression` is intentionally excluded — it is far too broad and
//! would flood the catalog. Seed: add a synthetic `nullish_coalescing` hint
//! kind when operator-level extraction is available.

/// Tree-sitter node kinds for null-safety usage.
///
/// Two node kinds are classified here:
///
/// - `optional_chain` — TypeScript and JavaScript optional chaining
///   (`a?.b`, `arr?.[0]`). One instance per `optional_chain` node as emitted
///   by the grammar. Note: optional calls (`fn?.()`) emit `call_expression`,
///   not `optional_chain` — they are not counted here.
/// - `non_null_expression` — TypeScript non-null assertion operator (`el!`).
///   TypeScript-only; not emitted by the JavaScript adapter.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::null_safety::NODE_KINDS;
///
/// assert!(NODE_KINDS.contains(&"optional_chain"));
/// assert!(NODE_KINDS.contains(&"non_null_expression"));
/// ```
pub const NODE_KINDS: &[&str] = &["optional_chain", "non_null_expression"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_kinds_contains_optional_chain() {
        assert!(NODE_KINDS.contains(&"optional_chain"));
    }

    #[test]
    fn node_kinds_contains_non_null_expression() {
        assert!(NODE_KINDS.contains(&"non_null_expression"));
    }

    #[test]
    fn node_kinds_has_two_entries() {
        assert_eq!(NODE_KINDS.len(), 2);
    }
}
