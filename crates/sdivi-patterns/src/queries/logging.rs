//! Node kinds that conceptually belong to the `logging` category.
//!
//! **This module is intentionally not wired into [`category_for_node_kind`].**
//! The node kinds listed below overlap with [`data_access`](super::data_access)
//! (`call_expression`, `call`) and [`resource_management`](super::resource_management)
//! (`macro_invocation`) at the AST level — only the callee name distinguishes
//! a logging invocation from a data-access or resource-management one. Native
//! classification by node kind alone would either steal hits from those
//! existing categories or duplicate-classify every call/macro.
//!
//! Foreign extractors (e.g. the Meridian consumer app) MUST apply callee-text
//! filtering on their side before emitting `PatternInstanceInput` values
//! with `category = "logging"`. The supported callee shapes per language are
//! documented in `docs/pattern-categories.md`.
//!
//! [`category_for_node_kind`]: super::category_for_node_kind

/// Conceptual tree-sitter node kinds for logging patterns. Reference only —
/// not consulted by [`category_for_node_kind`](super::category_for_node_kind).
///
/// Listed for documentation parity with sibling category modules and so
/// that embedders can grep `sdivi-patterns/src/queries/logging.rs` to see
/// the node-kind shapes the canonical contract has in mind.
///
/// - `call_expression`: TS/JS/Go logger / console / Print* calls
/// - `call`: Python `logging.*` and `print`
/// - `macro_invocation`: Rust `tracing::*!`, `log::*!`, `println!`/`eprintln!`
pub const NODE_KINDS: &[&str] = &["call_expression", "call", "macro_invocation"];
