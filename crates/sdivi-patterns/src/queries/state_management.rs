//! Node kinds classified as state-management patterns.
//!
//! These node kinds correspond to the `state_management` category in the
//! [`PatternCatalog`](crate::catalog::PatternCatalog).

/// Tree-sitter node kinds for state-management patterns.
///
/// - `closure_expression`: closures that capture mutable or shared state
pub const NODE_KINDS: &[&str] = &["closure_expression"];
