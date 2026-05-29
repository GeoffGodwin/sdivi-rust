//! Node kinds classified as data-access patterns.
//!
//! These node kinds correspond to the `data_access` category in the
//! [`PatternCatalog`](crate::catalog::PatternCatalog).

/// Tree-sitter node kinds for data-access patterns.
///
/// - `call_expression`: function/method calls that access data stores or
///   external resources (TypeScript/JavaScript/Go: `fetch`, `query`, `read`,
///   `write`, `db.*`, `sql.*`, etc.). All `call_expression` nodes are
///   classified here; callee-name narrowing is the consumer's responsibility.
/// - `call`: Python function calls accessing data (`cursor.*`, `session.*`,
///   `open`). Same broad-classification rule as above.
pub const NODE_KINDS: &[&str] = &["call_expression", "call"];
