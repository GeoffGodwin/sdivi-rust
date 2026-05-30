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
//! with `category = "logging"`. As of M32, [`matches_callee`] provides the
//! canonical regex tables — embedders should prefer it over hand-rolled filters.
//!
//! [`category_for_node_kind`]: super::category_for_node_kind

use std::sync::LazyLock;

use regex::Regex;

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

// TypeScript / JavaScript:
//   ^(console|logger|log)\.  — console.log, logger.info, log.debug, etc.
// ^ anchors to the start of the callee text.
static TS_JS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(console|logger|log)\.").expect("logging TS/JS regex is valid"));

// Python:
//   ^(logging\.|print\b)  — logging.info(...) or print(...); \b guards "printer" etc.
static PYTHON_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(logging\.|print\b)").expect("logging Python regex is valid"));

// Go:
//   ^fmt\.(Print|Println|Printf|Errorf|Fprint|Sprint)  — fmt package output calls.
// ^ anchor + exact package prefix prevents matching "myfmt.Println".
static GO_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^fmt\.(Print|Println|Printf|Errorf|Fprint|Sprint)")
        .expect("logging Go regex is valid")
});

// Rust:
//   ^(tracing|log)::  — tracing::info!("...") or log::debug!(...)
//   ^(println|eprintln|print|eprint|dbg)!  — standard macro forms
// Two anchored alternatives combined with |; both start with ^.
static RUST_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(tracing|log)::|^(println|eprintln|print|eprint|dbg)!")
        .expect("logging Rust regex is valid")
});

// Java:
//   ^(System\.(out|err)\.|logger\.|Log\.|LOG\.)  — common Java logging patterns.
// ^ anchor prevents matching "mySystem.out.println".
static JAVA_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(System\.(out|err)\.|logger\.|Log\.|LOG\.)").expect("logging Java regex is valid")
});

/// Return `true` when `text` looks like a logging callee for `language`.
///
/// This function promotes `logging` from catalog-only (M30 design) to natively
/// classifiable when called via [`classify_hint`](super::classify_hint). The
/// older [`category_for_node_kind`](super::category_for_node_kind) continues to
/// never return `Some("logging")` — that sentinel is unchanged.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::logging::matches_callee;
///
/// assert!(matches_callee("console.log(\"x\")", "typescript"));
/// assert!(matches_callee("tracing::info!(\"hi\")", "rust"));
/// assert!(matches_callee("fmt.Println(\"x\")", "go"));
/// assert!(!matches_callee("Math.max(a, b)", "typescript"));
/// assert!(!matches_callee("vec![1, 2]", "rust"));
/// ```
pub fn matches_callee(text: &str, language: &str) -> bool {
    match language {
        "typescript" | "javascript" => TS_JS_RE.is_match(text),
        "python" => PYTHON_RE.is_match(text),
        "go" => GO_RE.is_match(text),
        "rust" => RUST_RE.is_match(text),
        "java" => JAVA_RE.is_match(text),
        _ => false,
    }
}
