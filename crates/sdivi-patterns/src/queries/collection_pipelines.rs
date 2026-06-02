//! Callee-text classification for functional collection-transform method calls.
//!
//! Detects the standard array/iterable pipeline methods in TypeScript and JavaScript:
//! `.map`, `.filter`, `.reduce`, `.flatMap`, `.forEach`, `.find`, `.findIndex`,
//! `.some`, `.every`, `.flat`. These are the canonical functional-iteration primitives
//! and form a clean convention-drift signal — functional vs. imperative iteration is
//! one of the most visible style axes in JS codebases.
//!
//! The same regex applies to any language whose adapter collects `call_expression`
//! nodes with these method names (Go, Java), though those ecosystems rarely use
//! these exact names — TypeScript and JavaScript are the primary target.
//!
//! **Accepted noise:** Callee-text cannot distinguish an array `.map` from:
//! - `rxObservable.map(fn)` (RxJS)
//! - `new Map().forEach(cb)` (ES6 Map)
//! - `domNodeList.forEach(cb)` (DOM NodeList)
//! - `set.forEach(cb)` (ES6 Set)
//!
//! This is acceptable — the signal is the functional-iteration population at codebase
//! scale (entropy / convention drift), not the receiver type of each individual call.
//! Receiver-type inference would require a type-info pass that SDIVI deliberately
//! does not compute. Document the limitation rather than constraining the regex.
//!
//! **Pipe/compose seeds forward:** `pipe(...)`, `compose(...)`, `flow(...)` from
//! lodash/fp-ts/Ramda are the same idiom family and could extend this regex in a
//! future milestone.
//!
//! Detection is member-call callee-text only — no tree-sitter node-kind matching.
//! `call_expression` nodes are collected by the TS/JS adapters; this module provides
//! callee-text discrimination in `CALL_DISPATCH` at slot P10.

use std::sync::LazyLock;

use regex::Regex;

/// Tree-sitter node kinds for collection-pipeline patterns.
///
/// Empty — this category is detected entirely via callee-text inspection in
/// [`matches_callee`]. The `call_expression` node kind is already collected
/// by the TypeScript/JavaScript adapters; classification happens in
/// `classify_hint`'s `CALL_DISPATCH` loop at slot P10.
pub const NODE_KINDS: &[&str] = &[];

// TypeScript / JavaScript (and Go/Java where these method names appear) —
// member-call pattern: requires a preceding dot, so bare `map(f)` without a
// receiver is intentionally not matched. The named methods are:
//   map, filter, reduce, flatMap, forEach, find, findIndex, some, every, flat
//
// Disjointness:
//   - data_access:    \.(query|read|write|fetch)\(    — no token overlap
//   - async_patterns: \.(then|catch|finally)\(        — no token overlap
static TS_JS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\.(map|filter|reduce|flatMap|forEach|find|findIndex|some|every|flat)\(")
        .expect("collection_pipelines regex is valid")
});

/// Return `true` when `text` looks like a collection-pipeline callee for `language`.
///
/// Matches any language that emits `call_expression` nodes with these member calls
/// (TypeScript, JavaScript, and potentially Go/Java). Callee-text cannot distinguish
/// the receiver type — `rxObservable.map(fn)`, `new Map().forEach(cb)`, and
/// `array.map(f)` all match. This is intentional; see the module doc for the
/// receiver-type noise note.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::collection_pipelines::matches_callee;
///
/// assert!(matches_callee("xs.map(f)", "typescript"));
/// assert!(matches_callee("xs.filter(p).reduce(g, 0)", "typescript"));
/// assert!(matches_callee("data.flatMap(fn)", "javascript"));
/// assert!(matches_callee("items.forEach(cb)", "typescript"));
/// assert!(matches_callee("xs.find(p)", "typescript"));
/// assert!(matches_callee("xs.findIndex(p)", "javascript"));
/// assert!(matches_callee("xs.some(p)", "typescript"));
/// assert!(matches_callee("xs.every(p)", "typescript"));
/// assert!(matches_callee("xs.flat()", "javascript"));
/// // NOT matched: no dot prefix
/// assert!(!matches_callee("Math.max(a, b)", "typescript"));
/// // NOT matched: data_access methods
/// assert!(!matches_callee("db.query(sql)", "typescript"));
/// // NOT matched: async_patterns methods
/// assert!(!matches_callee("promise.then(resolve)", "typescript"));
/// ```
pub fn matches_callee(text: &str, language: &str) -> bool {
    match language {
        "typescript" | "javascript" | "go" | "java" => TS_JS_RE.is_match(text),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_matches() {
        assert!(matches_callee("xs.map(f)", "typescript"));
        assert!(matches_callee("data.map(fn)", "javascript"));
    }

    #[test]
    fn filter_matches() {
        assert!(matches_callee("xs.filter(p)", "typescript"));
    }

    #[test]
    fn reduce_matches() {
        assert!(matches_callee("xs.reduce(g, 0)", "typescript"));
    }

    #[test]
    fn flat_map_matches() {
        assert!(matches_callee("xs.flatMap(fn)", "javascript"));
    }

    #[test]
    fn for_each_matches() {
        assert!(matches_callee("xs.forEach(cb)", "typescript"));
    }

    #[test]
    fn find_matches() {
        assert!(matches_callee("xs.find(p)", "typescript"));
    }

    #[test]
    fn find_index_matches() {
        assert!(matches_callee("xs.findIndex(p)", "javascript"));
    }

    #[test]
    fn some_matches() {
        assert!(matches_callee("xs.some(p)", "typescript"));
    }

    #[test]
    fn every_matches() {
        assert!(matches_callee("xs.every(p)", "typescript"));
    }

    #[test]
    fn flat_matches() {
        assert!(matches_callee("xs.flat()", "javascript"));
    }

    #[test]
    fn chained_pipeline_matches() {
        // The first matching method wins; chained call text contains at least one.
        assert!(matches_callee("xs.filter(p).reduce(g, 0)", "typescript"));
    }

    #[test]
    fn data_access_methods_do_not_match() {
        // data_access uses query/read/write/fetch — disjoint
        assert!(!matches_callee("db.query(sql)", "typescript"));
        assert!(!matches_callee("client.read(buf)", "typescript"));
        assert!(!matches_callee("stream.write(data)", "typescript"));
        assert!(!matches_callee("client.fetch(url)", "typescript"));
    }

    #[test]
    fn async_pattern_methods_do_not_match() {
        // async_patterns uses then/catch/finally — disjoint
        assert!(!matches_callee("promise.then(resolve)", "typescript"));
        assert!(!matches_callee("p.catch(err => {})", "javascript"));
        assert!(!matches_callee("p.finally(() => {})", "typescript"));
    }

    #[test]
    fn math_max_does_not_match() {
        // Math.max has no collection-pipeline methods
        assert!(!matches_callee("Math.max(a, b)", "typescript"));
    }

    #[test]
    fn bare_function_call_does_not_match() {
        // No dot prefix — bare calls are intentionally excluded
        assert!(!matches_callee("map(f)", "typescript"));
        assert!(!matches_callee("filter(p)", "javascript"));
    }

    #[test]
    fn other_languages_return_false() {
        // Python and Rust adapters do not emit these callee forms
        for lang in ["python", "rust"] {
            assert!(
                !matches_callee("xs.map(f)", lang),
                "xs.map should not match for {lang}"
            );
        }
    }

    #[test]
    fn node_kinds_is_empty() {
        // NODE_KINDS is intentionally empty: this category is callee-only (classified
        // via classify_hint). The assertion guards that contract against regressions.
        #[allow(clippy::const_is_empty)]
        let empty = NODE_KINDS.is_empty();
        assert!(empty);
    }
}
