//! [`PatternHintInput`] — WASM-safe input struct for [`crate::queries::classify_hint`].

use serde::{Deserialize, Serialize};

/// A minimal hint struct for callee-text classification.
///
/// This is the pure-compute counterpart to `sdivi_parsing::feature_record::PatternHint`.
/// It contains only the two fields that [`crate::queries::classify_hint`] inspects:
/// the tree-sitter `node_kind` and the truncated source `text` of the node.
///
/// Foreign extractors (WASM consumers, Meridian) construct `PatternHintInput` directly.
/// The native pipeline uses `PatternHint` from `sdivi-parsing`; M33 will add a
/// conversion when the pipeline is wired to `classify_hint`.
///
/// `text` is truncated to 256 bytes upstream (per the `PatternHint` contract).
/// `classify_hint` matches only the callee prefix, so truncation never affects
/// classification correctness.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::PatternHintInput;
///
/// let hint = PatternHintInput {
///     node_kind: "call_expression".to_string(),
///     text: "console.log(\"hello\")".to_string(),
/// };
/// assert_eq!(hint.node_kind, "call_expression");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PatternHintInput {
    /// The tree-sitter node kind (e.g. `"call_expression"`, `"macro_invocation"`).
    pub node_kind: String,
    /// Source text of the node, truncated to 256 bytes if the original is longer.
    pub text: String,
}
