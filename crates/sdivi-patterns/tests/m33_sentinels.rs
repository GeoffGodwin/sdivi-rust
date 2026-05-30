//! M33 positive sentinels — lock in the `classify_hint` pipeline promotion.
//!
//! These tests encode that `classify_hint` DOES return `["logging"]` for the
//! callee shapes that the native pipeline now classifies natively (since M33).
//! A future contributor cannot revert the pipeline to `category_for_node_kind`
//! without tripping these invariants.
//!
//! Related: `queries::category_for_node_kind_never_returns_logging` (M30 sentinel,
//! in `queries/mod.rs` inline tests) captures the OLDER API's invariant; it stays
//! green because `category_for_node_kind` is unchanged. The present file captures
//! the M33 `classify_hint` invariant. Both must coexist.

use sdivi_patterns::queries::classify_hint;
use sdivi_patterns::PatternHintInput;

fn hint(node_kind: &str, text: &str) -> PatternHintInput {
    PatternHintInput {
        node_kind: node_kind.to_string(),
        text: text.to_string(),
    }
}

/// `console.log(...)` in TypeScript is classified as `logging` by the native
/// pipeline since M33. This test locks in that invariant.
#[test]
fn classify_hint_returns_logging_for_console_log() {
    assert_eq!(
        classify_hint(&hint("call_expression", "console.log(\"x\")"), "typescript"),
        vec!["logging"],
        "console.log must classify as logging — this is the M33 native pipeline invariant"
    );
}

/// `tracing::info!(...)` in Rust is classified as `logging` (not `resource_management`)
/// by the native pipeline since M33. This test locks in that invariant.
#[test]
fn classify_hint_returns_logging_for_tracing_macro() {
    assert_eq!(
        classify_hint(
            &hint("macro_invocation", "tracing::info!(\"request\")"),
            "rust"
        ),
        vec!["logging"],
        "tracing::info! must classify as logging — this is the M33 native pipeline invariant"
    );
}

/// Unrecognised callees (e.g. `Math.max(a, b)`) return an empty Vec from `classify_hint`.
/// The hint is silently dropped — same behaviour as the prior `None` path from
/// `category_for_node_kind`.
#[test]
fn classify_hint_drops_unrecognised_calls() {
    assert!(
        classify_hint(&hint("call_expression", "Math.max(a, b)"), "typescript").is_empty(),
        "Math.max must return empty Vec — unrecognised callees are silently dropped"
    );
}
