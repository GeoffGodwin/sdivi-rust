//! Node kinds classified as async-concurrency patterns.
//!
//! These node kinds correspond to the `async_patterns` category in the
//! [`PatternCatalog`](crate::catalog::PatternCatalog).

use std::sync::LazyLock;

use regex::Regex;

/// Tree-sitter node kinds for async-concurrency patterns.
///
/// - `await_expression`: `.await` on a `Future`
pub const NODE_KINDS: &[&str] = &["await_expression"];

// TypeScript / JavaScript:
//   \.(then|catch|finally)\(  — Promise chain calls.
// No ^ anchor: the receiver expression precedes the dot, so the match is
// suffix-anchored to the method name. This deliberately matches
// "promise.then(" and "fetch(...).then(" without matching "getNextValue(".
static TS_JS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\.(then|catch|finally)\(").expect("async_patterns TS/JS regex is valid")
});

/// Return `true` when `text` looks like an async-pattern call callee for `language`.
///
/// Covers Promise-chain `call_expression` shapes (`.then`, `.catch`, `.finally`)
/// in TypeScript and JavaScript. `await_expression` nodes are routed via
/// [`NODE_KINDS`] already; this function handles the remaining callee shapes.
/// Other languages return `false` — their async primitives are node-kind-routed.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::async_patterns::matches_callee;
///
/// assert!(matches_callee("promise.then(resolve)", "typescript"));
/// assert!(matches_callee("fetch(url).catch(err => {})", "javascript"));
/// assert!(!matches_callee("Math.max(a, b)", "typescript"));
/// assert!(!matches_callee("promise.then(resolve)", "rust"));
/// ```
pub fn matches_callee(text: &str, language: &str) -> bool {
    match language {
        "typescript" | "javascript" => TS_JS_RE.is_match(text),
        _ => false,
    }
}
