//! Callee-text classification for test-suite structure and assertion patterns.
//!
//! Detects test-framework globals and helpers:
//!
//! - **TypeScript / JavaScript:** BDD globals (`describe`, `it`, `context`), flat `test`,
//!   lifecycle hooks (`beforeEach`, `afterEach`, `beforeAll`, `afterAll`), `expect(…)`,
//!   focused/excluded variants (`xit`, `xdescribe`, `fit`, `fdescribe`), and
//!   framework-namespaced helpers (`jest.fn`, `jest.mock`, `jest.spyOn`, `vi.fn`, `vi.mock`,
//!   `vi.spyOn`, etc.). Covers Jest, Vitest, Mocha, and Jasmine globals.
//! - **Go:** `testing.T` method calls — `t.Run`, `t.Error`, `t.Errorf`, `t.Fatal`,
//!   `t.Fatalf`, `t.Helper`, `t.Skip`, `t.Skipf`, `t.Log`, `t.Logf`, `t.Cleanup`,
//!   `t.Parallel`.
//! - **Python:** `unittest.TestCase` assertion methods — `self.assertEqual`,
//!   `self.assertTrue`, and the full `self.assert[A-Z]…` family. pytest bare `assert`
//!   statements are not call nodes and produce no hits in the v0 model.
//!
//! ## CALL_DISPATCH slot
//!
//! Registered at P2 — just below `async_patterns` (P1) and above `schema_validation`
//! (P4). Test globals are specific and resolve before any broader category.
//!
//! ## Known false positives
//!
//! `test(args)`, `it(args)`, `context(args)`, and `expect(x)` can appear as business-logic
//! function names. The `^` anchor prevents mid-identifier matching but cannot distinguish
//! intent. Accepted as entropy noise at codebase scale. Exclude test paths via
//! `patterns.scope_exclude` to suppress them.
//!
//! ## `scope_exclude` interaction
//!
//! `patterns.scope_exclude` removes files from the pattern catalog only (files remain in the
//! graph). The `testing` bucket is non-empty only when test files are in the pattern scope.
//! Repos that exclude test paths via `scope_exclude` will see a zero count — intended
//! behaviour, not a miss. No auto-detection of test paths is performed.
//!
//! ## Seeds forward
//!
//! Property-based (`fc.assert`, `hypothesis.given`) and E2E frameworks (`cy.`, `page.`)
//! are adjacent idioms. Deferred to post-M42.

use std::sync::LazyLock;

use regex::Regex;

/// Tree-sitter node kinds for testing patterns.
///
/// Empty — detection is entirely via callee-text inspection in [`matches_callee`].
/// Classification happens in `classify_hint`'s `CALL_DISPATCH` loop at slot P2.
pub const NODE_KINDS: &[&str] = &[];

// TypeScript / JavaScript — BDD globals, flat test, lifecycle hooks, expect root.
// Anchored at `^`; covers Jest/Vitest/Mocha/Jasmine/Qunit test-suite globals.
static TS_JS_GLOBALS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(describe|it|test|xit|xdescribe|fdescribe|fit|context|beforeEach|afterEach|beforeAll|afterAll|expect)\(",
    )
    .expect("testing TS/JS globals regex is valid")
});

// TypeScript / JavaScript — Jest and Vitest framework-namespaced helpers.
static TS_JS_FRAMEWORK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(jest|vi)\.(fn|mock|spyOn|clearAllMocks|resetAllMocks|useFakeTimers)\(")
        .expect("testing TS/JS framework regex is valid")
});

// Go — testing.T method calls.
// `\bt\.` requires a word boundary before `t`; receiver `st` does not match.
static GO_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\bt\.(Run|Error|Errorf|Fatal|Fatalf|Helper|Skip|Skipf|Log|Logf|Cleanup|Parallel)\(",
    )
    .expect("testing Go regex is valid")
});

// Python — unittest.TestCase assertion methods.
// `self.assert[A-Z]…(` rules out `self.assert_` snake_case helpers.
static PYTHON_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bself\.assert[A-Z]\w*\(").expect("testing Python regex is valid")
});

/// Return `true` when `text` looks like a test-framework call.
///
/// Covers BDD globals, flat `test`, lifecycle hooks, `expect` roots (TS/JS),
/// `testing.T` methods (Go), and `unittest.TestCase` assertion methods (Python).
/// See module doc for scope_exclude interaction and false-positive policy.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::testing::matches_callee;
///
/// assert!(matches_callee("describe('suite', fn)", "typescript"));
/// assert!(matches_callee("it('does', fn)", "javascript"));
/// assert!(matches_callee("expect(x).toBe(1)", "javascript"));
/// assert!(matches_callee("beforeEach(() => {})", "typescript"));
/// assert!(matches_callee("jest.mock('./module')", "typescript"));
/// assert!(matches_callee("vi.fn()", "javascript"));
/// assert!(matches_callee("t.Run(\"sub\", fn)", "go"));
/// assert!(matches_callee("t.Fatal(err)", "go"));
/// assert!(matches_callee("self.assertEqual(a, b)", "python"));
/// assert!(matches_callee("self.assertTrue(x)", "python"));
/// assert!(!matches_callee("console.log(x)", "typescript"));
/// assert!(!matches_callee("db.Query(sql)", "go"));
/// assert!(!matches_callee("self.method()", "python"));
/// ```
pub fn matches_callee(text: &str, language: &str) -> bool {
    match language {
        "typescript" | "javascript" => {
            TS_JS_GLOBALS_RE.is_match(text) || TS_JS_FRAMEWORK_RE.is_match(text)
        }
        "go" => GO_RE.is_match(text),
        "python" => PYTHON_RE.is_match(text),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_matches_ts() {
        assert!(matches_callee("describe('suite', fn)", "typescript"));
    }

    #[test]
    fn it_matches_js() {
        assert!(matches_callee("it('does something', fn)", "javascript"));
    }

    #[test]
    fn test_matches_ts() {
        assert!(matches_callee("test('is truthy', () => {})", "typescript"));
    }

    #[test]
    fn expect_matches_js() {
        assert!(matches_callee("expect(x).toBe(1)", "javascript"));
    }

    #[test]
    fn before_each_matches() {
        assert!(matches_callee("beforeEach(() => {})", "typescript"));
    }

    #[test]
    fn after_all_matches() {
        assert!(matches_callee("afterAll(() => {})", "javascript"));
    }

    #[test]
    fn xit_matches() {
        assert!(matches_callee("xit('skipped', fn)", "typescript"));
    }

    #[test]
    fn context_matches() {
        assert!(matches_callee("context('ctx', fn)", "javascript"));
    }

    #[test]
    fn jest_mock_matches() {
        assert!(matches_callee("jest.mock('./module')", "typescript"));
    }

    #[test]
    fn jest_fn_matches() {
        assert!(matches_callee("jest.fn()", "typescript"));
    }

    #[test]
    fn jest_spy_on_matches() {
        assert!(matches_callee("jest.spyOn(obj, 'method')", "typescript"));
    }

    #[test]
    fn vi_fn_matches() {
        assert!(matches_callee("vi.fn()", "javascript"));
    }

    #[test]
    fn vi_mock_matches() {
        assert!(matches_callee("vi.mock('./mod')", "typescript"));
    }

    #[test]
    fn console_log_does_not_match() {
        assert!(!matches_callee("console.log(x)", "typescript"));
    }

    #[test]
    fn use_effect_does_not_match() {
        assert!(!matches_callee("useEffect(fn, [])", "typescript"));
    }

    #[test]
    fn t_run_matches_go() {
        assert!(matches_callee("t.Run(\"sub\", fn)", "go"));
    }

    #[test]
    fn t_fatal_matches_go() {
        assert!(matches_callee("t.Fatal(err)", "go"));
    }

    #[test]
    fn t_parallel_matches_go() {
        assert!(matches_callee("t.Parallel()", "go"));
    }

    #[test]
    fn go_st_run_does_not_match() {
        // `st.Run(...)` — `t` preceded by `s` (word char); `\bt\.` does not match.
        assert!(!matches_callee("st.Run(\"sub\", fn)", "go"));
    }

    #[test]
    fn self_assert_equal_matches() {
        assert!(matches_callee("self.assertEqual(a, b)", "python"));
    }

    #[test]
    fn self_assert_true_matches() {
        assert!(matches_callee("self.assertTrue(x)", "python"));
    }

    #[test]
    fn self_method_does_not_match() {
        assert!(!matches_callee("self.method()", "python"));
    }

    #[test]
    fn self_assert_snake_does_not_match() {
        assert!(!matches_callee("self.assert_something()", "python"));
    }

    #[test]
    fn rust_returns_false() {
        assert!(!matches_callee("describe('s', fn)", "rust"));
    }

    #[test]
    fn node_kinds_is_empty() {
        assert!(NODE_KINDS.is_empty());
    }
}
