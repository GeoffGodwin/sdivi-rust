//! Node kinds classified as async-concurrency patterns.
//!
//! These node kinds correspond to the `async_patterns` category in the
//! [`PatternCatalog`](crate::catalog::PatternCatalog).

/// Tree-sitter node kinds for async-concurrency patterns.
///
/// - `await_expression`: `.await` on a `Future`
pub const NODE_KINDS: &[&str] = &["await_expression"];
