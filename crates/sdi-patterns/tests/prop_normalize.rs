//! Property test: `normalize_and_hash` is stable (same input → same digest).
//!
//! This guards the contract that foreign extractors (consumer app, WASM callers)
//! can rely on fingerprints being identical across runs for the same AST structure.

use proptest::prelude::*;
use sdi_patterns::normalize::{NormalizeNode, normalize_and_hash};

fn arb_node_kind() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[a-z_]{3,20}").unwrap()
}

fn arb_leaf() -> impl Strategy<Value = NormalizeNode> {
    arb_node_kind().prop_map(|kind| NormalizeNode { kind, children: vec![] })
}

fn arb_node(depth: usize) -> impl Strategy<Value = NormalizeNode> {
    if depth == 0 {
        arb_leaf().boxed()
    } else {
        (arb_node_kind(), proptest::collection::vec(arb_node(depth - 1), 0..4))
            .prop_map(|(kind, children)| NormalizeNode { kind, children })
            .boxed()
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// `normalize_and_hash` is stable: same kind + children → same hex string.
    #[test]
    fn prop_test_normalize_and_hash_stable(
        node in arb_node(3),
    ) {
        let h1 = normalize_and_hash(&node.kind, &node.children);
        let h2 = normalize_and_hash(&node.kind, &node.children);
        prop_assert_eq!(h1, h2, "same inputs must produce same blake3 digest");
    }

    /// Result is always a 64-character lowercase hex string.
    #[test]
    fn prop_result_is_64_char_hex(node in arb_node(2)) {
        let h = normalize_and_hash(&node.kind, &node.children);
        prop_assert_eq!(h.len(), 64, "digest must be 64 hex chars");
        prop_assert!(h.chars().all(|c| c.is_ascii_hexdigit()), "must be hex");
    }

    /// Different kinds produce different digests (collision resistance sanity check).
    #[test]
    fn prop_different_kinds_differ(
        k1 in proptest::string::string_regex("[a-z]{4,12}").unwrap(),
        k2 in proptest::string::string_regex("[a-z]{4,12}").unwrap(),
    ) {
        prop_assume!(k1 != k2);
        let h1 = normalize_and_hash(&k1, &[]);
        let h2 = normalize_and_hash(&k2, &[]);
        prop_assert_ne!(h1, h2, "different kinds must produce different digests");
    }
}
