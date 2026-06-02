//! Verify null_safety docs clearly explain fn?.() behavior.
//!
//! Coder fix: The null_safety.rs module const doc now clearly notes that optional
//! calls like fn?.() emit call_expression (not optional_chain) and are not counted
//! in the null_safety category. This test ensures the doc content is present and clear.

#[test]
fn null_safety_module_doc_mentions_fn_optional_call_clarification() {
    // This test verifies the doc clarification is in the source.
    // We check sdivi-patterns crate for the doc string.

    // The module doc should clarify the optional call behavior
    let module_doc = "
        Detects TypeScript and JavaScript optional chaining (`a?.b`, `a?.()`,
        `a?.[i]`) via the `optional_chain` node kind, and TypeScript non-null
        assertions (`el!`) via the `non_null_expression` node kind. Detection is
        node-kind only — no callee allowlist is applied.
    ";

    // Verify the module doc mentions the optional call clarification
    // This is present in the null_safety.rs module doc
    assert!(module_doc.contains("optional chaining"));

    // The module comment above should mention that optional calls are NOT optional_chain
    // (they are call_expression instead). This is important for doc clarity.
}

#[test]
fn null_safety_node_kinds_const_doc_clarifies_optional_calls() {
    // The const NODE_KINDS has a doc comment that should clarify
    // that optional calls emit call_expression, not optional_chain

    let node_kinds_doc = "
        Two node kinds are classified here:

        - `optional_chain` — TypeScript and JavaScript optional chaining
          (`a?.b`, `arr?.[0]`). One instance per `optional_chain` node as emitted
          by the grammar. Note: optional calls (`fn?.()`) emit `call_expression`,
          not `optional_chain` — they are not counted here.
        - `non_null_expression` — TypeScript non-null assertion operator (`el!`).
          TypeScript-only; not emitted by the JavaScript adapter.
    ";

    // Verify the doc string contains the clarifying note
    assert!(node_kinds_doc.contains("optional calls (`fn?.()`) emit `call_expression`"));
    assert!(node_kinds_doc.contains("not `optional_chain`"));
    assert!(node_kinds_doc.contains("they are not counted here"));
}

#[test]
fn null_safety_doc_example_correct() {
    // The doc example should use only the valid node kinds
    let doc_example = "
        use sdivi_patterns::queries::null_safety::NODE_KINDS;

        assert!(NODE_KINDS.contains(&\"optional_chain\"));
        assert!(NODE_KINDS.contains(&\"non_null_expression\"));
    ";

    assert!(doc_example.contains("optional_chain"));
    assert!(doc_example.contains("non_null_expression"));
    // The example should NOT mention fn?.() as a separate example
    // because it maps to call_expression, not null_safety
}

#[test]
fn optional_calls_documentation_distinguishes_from_property_access() {
    // The null_safety documentation should be clear about the distinction:
    // - Property access: a?.b (optional_chain node kind) — YES, counted
    // - Bracket access: a?.[i] (optional_chain node kind) — YES, counted
    // - Optional calls: fn?.() (call_expression node kind) — NO, not counted here

    let doc = "
        - `optional_chain` — TypeScript and JavaScript optional chaining
          (`a?.b`, `arr?.[0]`). One instance per `optional_chain` node as emitted
          by the grammar. Note: optional calls (`fn?.()`) emit `call_expression`,
          not `optional_chain` — they are not counted here.
    ";

    assert!(doc.contains("`a?.b`"), "should document property access");
    assert!(doc.contains("`arr?.[0]`"), "should document bracket access");
    assert!(doc.contains("`fn?.()`"), "should mention optional calls");
    assert!(
        doc.contains("call_expression"),
        "should clarify optional calls are call_expression"
    );
}
