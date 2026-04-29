//! Node kinds classified as type-assertion/cast patterns.
//!
//! These node kinds correspond to the `type_assertions` category in the
//! [`PatternCatalog`](crate::catalog::PatternCatalog).

/// Tree-sitter node kinds for type-assertion patterns.
///
/// - `as_expression`: `expr as Type` casts
/// - `type_cast_expression`: language-specific cast expressions (e.g. TypeScript)
pub const NODE_KINDS: &[&str] = &["as_expression", "type_cast_expression"];
