//! Supplemental corpus for entries trimmed from `dispatch_disjointness.rs` at M43.
//!
//! `dispatch_disjointness.rs` hit the 300-line ceiling during M43 and had to drop:
//! - 5 collection_pipelines corpus entries (reduce, some, every, flat, findIndex)
//! - 2 Go logging corpus entries (fmt.Print, fmt.Errorf)
//! - 1 unrecognised callee entry
//!
//! This file acts as the "record" the reviewer flagged as missing, and also
//! exercises code paths that have no surviving representative in the main corpus.

use sdivi_patterns::queries::classify_hint;
use sdivi_patterns::PatternHintInput;

fn hint(node_kind: &str, text: &str) -> PatternHintInput {
    PatternHintInput {
        node_kind: node_kind.to_string(),
        text: text.to_string(),
    }
}

// ── collection_pipelines — methods not represented in the main corpus ─────────

#[test]
fn reduce_is_collection_pipelines() {
    let r = classify_hint(&hint("call_expression", "xs.reduce(g, 0)"), "typescript");
    assert_eq!(r, vec!["collection_pipelines"]);
}

#[test]
fn some_is_collection_pipelines() {
    let r = classify_hint(&hint("call_expression", "arr.some(p)"), "typescript");
    assert_eq!(r, vec!["collection_pipelines"]);
}

#[test]
fn every_is_collection_pipelines() {
    let r = classify_hint(&hint("call_expression", "arr.every(p)"), "typescript");
    assert_eq!(r, vec!["collection_pipelines"]);
}

#[test]
fn flat_is_collection_pipelines() {
    let r = classify_hint(&hint("call_expression", "arr.flat()"), "javascript");
    assert_eq!(r, vec!["collection_pipelines"]);
}

#[test]
fn find_index_is_collection_pipelines() {
    let r = classify_hint(&hint("call_expression", "arr.findIndex(p)"), "javascript");
    assert_eq!(r, vec!["collection_pipelines"]);
}

// ── Go logging — fmt variants not represented in the main corpus ──────────────

#[test]
fn fmt_print_is_logging_go() {
    let r = classify_hint(&hint("call_expression", "fmt.Print(\"x\")"), "go");
    assert_eq!(r, vec!["logging"]);
}

#[test]
fn fmt_errorf_is_logging_go() {
    // fmt.Errorf is a Go logging/diagnostic call, matched by the Go logging regex.
    let r = classify_hint(&hint("call_expression", "fmt.Errorf(\"err: %v\", err)"), "go");
    assert_eq!(r, vec!["logging"]);
}

// ── Unrecognised callee — classify_hint returns empty Vec ─────────────────────

#[test]
fn unrecognised_callee_returns_empty() {
    // A callee that matches no CALL_DISPATCH entry must yield an empty Vec.
    // Math.sqrt has no dot-method overlap with collection_pipelines and is not
    // a logging/async/data-access/serialization callee — genuinely unrecognised.
    let r = classify_hint(&hint("call_expression", "Math.sqrt(x)"), "typescript");
    assert!(
        r.is_empty(),
        "Math.sqrt must resolve to empty Vec; got {r:?}"
    );
}
