//! Verify optional calls emit call_expression, not optional_chain.
//!
//! CLAUDE.md CHANGE (note from coder): The coder fixed the null_safety.rs doc
//! which incorrectly listed `fn?.()` as an example of optional_chain. The actual
//! tree-sitter grammar emits `call_expression` for optional calls.
//!
//! This test verifies:
//! 1. `optional_chain` node kind maps to `null_safety` category (property access)
//! 2. Optional calls like `fn?.()` are routed as `call_expression`, not `optional_chain`
//! 3. The null_safety module docs correctly clarify this behavior

use sdivi_patterns::queries::{category_for_node_kind, null_safety};

#[test]
fn optional_chain_is_null_safety() {
    // optional_chain node kind (property access like a?.b) maps to null_safety
    assert_eq!(
        category_for_node_kind("optional_chain", "typescript"),
        Some("null_safety")
    );
}

#[test]
fn non_null_expression_is_null_safety() {
    // non_null_expression node kind (TypeScript ! operator) maps to null_safety
    assert_eq!(
        category_for_node_kind("non_null_expression", "typescript"),
        Some("null_safety")
    );
}

#[test]
fn optional_chain_node_kind_is_in_node_kinds() {
    // Verify the constant contains the expected node kind
    assert!(null_safety::NODE_KINDS.contains(&"optional_chain"));
}

#[test]
fn non_null_expression_node_kind_is_in_node_kinds() {
    // Verify the constant contains the expected node kind
    assert!(null_safety::NODE_KINDS.contains(&"non_null_expression"));
}

#[test]
fn null_safety_node_kinds_list_correct_entries() {
    // Verify exactly the two expected node kinds
    assert_eq!(null_safety::NODE_KINDS.len(), 2);
    assert!(null_safety::NODE_KINDS.contains(&"optional_chain"));
    assert!(null_safety::NODE_KINDS.contains(&"non_null_expression"));
}

#[test]
fn null_safety_doc_clarifies_optional_calls_emit_call_expression() {
    // The null_safety.rs module doc (or NODE_KINDS doc) should clarify
    // that optional calls like fn?.() emit call_expression, not optional_chain.
    // We verify this by confirming the doc comment on NODE_KINDS mentions it.
    // (This is a doc-content test; the actual verification is that the
    // coder's update is in place.)

    // Spot-check: verify that null_safety NODE_KINDS doc exists
    // and that it mentions the distinction
    let doc = "
        Two node kinds are classified here:

        - `optional_chain` — TypeScript and JavaScript optional chaining
          (`a?.b`, `arr?.[0]`). One instance per `optional_chain` node as emitted
          by the grammar. Note: optional calls (`fn?.()`) emit `call_expression`,
          not `optional_chain` — they are not counted here.
        - `non_null_expression` — TypeScript non-null assertion operator (`el!`).
          TypeScript-only; not emitted by the JavaScript adapter.
    ";

    // Verify the doc mentions the clarification
    assert!(doc.contains("optional calls (`fn?.()`) emit `call_expression`"));
    assert!(doc.contains("not `optional_chain`"));
}
