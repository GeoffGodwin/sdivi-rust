use sdivi_core::input::NormalizeNode;
use sdivi_core::normalize_and_hash;
use sdivi_patterns::fingerprint::fingerprint_node_kind;

/// M07-equivalence: empty children must match fingerprint_node_kind byte-for-byte.
#[test]
fn leaf_matches_fingerprint_node_kind_m07_equivalence() {
    for kind in &[
        "try_expression",
        "function_definition",
        "class_definition",
        "import_statement",
    ] {
        let expected = fingerprint_node_kind(kind).to_hex();
        let actual = normalize_and_hash(kind, &[]);
        assert_eq!(actual, expected, "M07 equivalence failed for kind={kind}");
    }
}

#[test]
fn returns_64_char_hex() {
    let h = normalize_and_hash("fn_item", &[]);
    assert_eq!(h.len(), 64);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn different_root_kinds_differ() {
    let a = normalize_and_hash("kind_a", &[]);
    let b = normalize_and_hash("kind_b", &[]);
    assert_ne!(a, b);
}

#[test]
fn nested_differs_from_flat() {
    let child = NormalizeNode {
        kind: "child".to_string(),
        children: vec![],
    };
    let with_child = normalize_and_hash("parent", &[child]);
    let without = normalize_and_hash("parent", &[]);
    assert_ne!(with_child, without);
}

#[test]
fn child_order_matters() {
    let c1 = NormalizeNode {
        kind: "a".to_string(),
        children: vec![],
    };
    let c2 = NormalizeNode {
        kind: "b".to_string(),
        children: vec![],
    };
    let h1 = normalize_and_hash("root", &[c1.clone(), c2.clone()]);
    let h2 = normalize_and_hash("root", &[c2, c1]);
    assert_ne!(h1, h2);
}

#[test]
fn deterministic_across_calls() {
    let child = NormalizeNode {
        kind: "body".to_string(),
        children: vec![],
    };
    let h1 = normalize_and_hash("function", &[child.clone()]);
    let h2 = normalize_and_hash("function", &[child]);
    assert_eq!(h1, h2);
}
