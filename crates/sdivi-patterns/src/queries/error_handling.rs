//! Node kinds classified as error-handling patterns.
//!
//! These node kinds correspond to the `error_handling` category in the
//! [`PatternCatalog`](crate::catalog::PatternCatalog).

/// Tree-sitter node kinds for error-handling patterns.
///
/// - `try_expression`: the `?` operator applied to a `Result` or `Option`
/// - `match_expression`: `match` arms used for result/error dispatch
pub const NODE_KINDS: &[&str] = &["try_expression", "match_expression"];
