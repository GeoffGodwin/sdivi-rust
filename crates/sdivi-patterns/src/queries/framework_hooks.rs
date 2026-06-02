//! Node kinds classified as framework-hook calls.
//!
//! Detects React, Preact, Vue (composables), and Svelte-style hook calls
//! in TypeScript and JavaScript. Identified by callee text (`^use[A-Z]`).
//! No tree-sitter node-kind matching — `call_expression` is already collected
//! by the TS/JS adapters; this module provides callee-text discrimination only.

use std::sync::LazyLock;

use regex::Regex;

/// Tree-sitter node kinds for framework-hook calls.
///
/// Empty — this category is detected entirely via callee-text inspection in
/// [`matches_callee`]. The `call_expression` node kind is already collected
/// by the TypeScript/JavaScript adapters; classification happens in
/// `classify_hint`'s `CALL_DISPATCH` loop at slot P6.
pub const NODE_KINDS: &[&str] = &[];

// TypeScript / JavaScript:
//   ^use[A-Z]  — any callee starting with `use` followed immediately by an
// uppercase letter. Covers the entire React / Vue / Svelte hook ecosystem:
// built-in hooks (useState, useEffect, useMemo, useCallback, useRef,
// useContext, useReducer, useLayoutEffect) and the full custom-hook ecosystem.
// The uppercase second character is mandatory — `user(x)`, `useful(x)` do NOT
// match; only `useX…` where X ∈ [A-Z] matches.
static TS_JS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^use[A-Z]").expect("framework_hooks TS/JS regex is valid"));

/// Return `true` when `text` looks like a framework-hook callee for `language`.
///
/// Matches any callee starting with `use` followed by an uppercase letter
/// (`useState`, `useEffect`, `useMemo`, `useAuth`, etc.) in TypeScript or
/// JavaScript. All other languages return `false` — hook conventions are
/// specific to the JS/TS ecosystem.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::framework_hooks::matches_callee;
///
/// assert!(matches_callee("useState(0)", "typescript"));
/// assert!(matches_callee("useEffect(fn, [])", "typescript"));
/// assert!(matches_callee("useAuth()", "javascript"));
/// assert!(matches_callee("useCustomHook(opts)", "typescript"));
/// assert!(!matches_callee("user()", "typescript"));     // lowercase second char
/// assert!(!matches_callee("getUser()", "typescript"));  // doesn't start with `use`
/// assert!(!matches_callee("useState(0)", "rust"));      // wrong language
/// ```
pub fn matches_callee(text: &str, language: &str) -> bool {
    match language {
        "typescript" | "javascript" => TS_JS_RE.is_match(text),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_hooks_match_typescript() {
        for callee in [
            "useState(0)",
            "useEffect(fn, [])",
            "useMemo(() => val, [])",
            "useCallback(fn, [])",
            "useRef(null)",
            "useContext(MyCtx)",
            "useReducer(reducer, state)",
            "useLayoutEffect(fn, [])",
        ] {
            assert!(
                matches_callee(callee, "typescript"),
                "{callee:?} should match for typescript"
            );
        }
    }

    #[test]
    fn custom_hooks_match() {
        assert!(matches_callee("useAuth()", "typescript"));
        assert!(matches_callee("useStore()", "javascript"));
        assert!(matches_callee("useCustomThing(opts)", "typescript"));
        assert!(matches_callee("useMutation(fn)", "javascript"));
    }

    #[test]
    fn lowercase_second_char_does_not_match() {
        assert!(!matches_callee("user()", "typescript"));
        assert!(!matches_callee("useful(x)", "javascript"));
        assert!(!matches_callee("username()", "typescript"));
        assert!(!matches_callee("fuse(x)", "typescript"));
    }

    #[test]
    fn non_use_prefix_does_not_match() {
        assert!(!matches_callee("getUser()", "typescript"));
        assert!(!matches_callee("setState(x)", "typescript"));
        assert!(!matches_callee("Math.max(a, b)", "typescript"));
        assert!(!matches_callee("console.log(x)", "typescript"));
        assert!(!matches_callee("fetch(url)", "typescript"));
    }

    #[test]
    fn other_languages_return_false() {
        for lang in ["rust", "python", "go", "java"] {
            assert!(
                !matches_callee("useState(0)", lang),
                "useState should not match for {lang}"
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
