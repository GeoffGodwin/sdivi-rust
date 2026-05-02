//! Tree-aware canonical fingerprint algorithm.
//!
//! Implements `normalize_and_hash` — the depth-first canonical walk that extends
//! the leaf-only `fingerprint_node_kind` to trees of arbitrary depth.

use crate::fingerprint::{PatternFingerprint, FINGERPRINT_KEY};

/// A node in a pattern AST subtree used by [`normalize_and_hash`].
///
/// A leaf node has `kind` set and an empty `children` slice.  For v0, language
/// adapters still emit kind-only pattern hints, so `children` is always empty
/// and the output is byte-identical to [`crate::fingerprint::fingerprint_node_kind`].
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::normalize::{NormalizeNode, normalize_and_hash};
///
/// // Leaf node: same digest as fingerprint_node_kind("try_expression")
/// let result = normalize_and_hash("try_expression", &[]);
/// assert_eq!(result.len(), 64);
/// ```
#[derive(Debug, Clone)]
pub struct NormalizeNode {
    /// The tree-sitter node kind string.
    pub kind: String,
    /// Ordered children of this node.
    pub children: Vec<NormalizeNode>,
}

/// Computes a canonical `blake3` fingerprint for a pattern AST subtree.
///
/// ## Algorithm
///
/// Depth-first canonical walk.  For a node with `kind = K` and ordered
/// children `[c1, c2, …]`, the input to `blake3::keyed_hash(FINGERPRINT_KEY, _)`
/// is the byte concatenation of:
/// - `K` (UTF-8 bytes)
/// - the separator byte `0x00`
/// - for each child: the byte `0x01` followed by the 32 digest bytes of the
///   recursively-computed child fingerprint
///
/// The `0x00` / `0x01` framing prevents a leaf node `K` from colliding with
/// an internal node `K` whose only child starts with `K`.
///
/// **Empty `children`** produces the same digest as
/// [`crate::fingerprint::fingerprint_node_kind`]`(kind)`:
/// `blake3::keyed_hash(FINGERPRINT_KEY, kind.as_bytes())` (the `0x00` suffix
/// on an empty children list makes the input identical to the raw kind bytes
/// followed by `0x00`, but `fingerprint_node_kind` feeds only `kind.as_bytes()`
/// — so they must be byte-identical).
///
/// Wait — to maintain equivalence with `fingerprint_node_kind(kind)` for empty
/// children, the algorithm for empty children uses `kind.as_bytes()` directly
/// (no separator, no framing), matching `fingerprint_node_kind`.
///
/// Returns a 64-character lowercase hex string.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::normalize::normalize_and_hash;
/// use sdivi_patterns::fingerprint::fingerprint_node_kind;
///
/// // Empty children must match fingerprint_node_kind.
/// let fp1 = fingerprint_node_kind("try_expression").to_hex();
/// let fp2 = normalize_and_hash("try_expression", &[]);
/// assert_eq!(fp1, fp2);
/// ```
pub fn normalize_and_hash(kind: &str, children: &[NormalizeNode]) -> String {
    let fp = normalize_internal(kind, children);
    fp.to_hex()
}

fn normalize_internal(kind: &str, children: &[NormalizeNode]) -> PatternFingerprint {
    if children.is_empty() {
        // Byte-identical to fingerprint_node_kind(kind): blake3::keyed_hash(KEY, kind.as_bytes())
        let hash = blake3::keyed_hash(&FINGERPRINT_KEY, kind.as_bytes());
        return PatternFingerprint::from_bytes(*hash.as_bytes());
    }

    // Internal node: K + 0x00 + (0x01 + child_bytes)*
    let mut input = Vec::with_capacity(kind.len() + 1 + children.len() * 33);
    input.extend_from_slice(kind.as_bytes());
    input.push(0x00u8);
    for child in children {
        let child_fp = normalize_internal(&child.kind, &child.children);
        input.push(0x01u8);
        input.extend_from_slice(child_fp.as_bytes());
    }

    let hash = blake3::keyed_hash(&FINGERPRINT_KEY, &input);
    PatternFingerprint::from_bytes(*hash.as_bytes())
}
