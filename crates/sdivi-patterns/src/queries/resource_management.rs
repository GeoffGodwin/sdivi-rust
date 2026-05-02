//! Node kinds classified as resource-management patterns.
//!
//! These node kinds correspond to the `resource_management` category in the
//! [`PatternCatalog`](crate::catalog::PatternCatalog).

/// Tree-sitter node kinds for resource-management patterns.
///
/// - `macro_invocation`: macro calls (e.g. `drop!`, `vec!`, `println!`)
pub const NODE_KINDS: &[&str] = &["macro_invocation"];
