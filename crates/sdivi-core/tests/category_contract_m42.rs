//! M42 acceptance-criterion tests: `testing` pattern category.
//!
//! Verifies that `classify_hint` routes test-framework calls to `["testing"]`
//! and that non-test calls are not miscategorised.

use sdivi_patterns::queries::classify_hint;
use sdivi_patterns::PatternHintInput;

fn hint(node_kind: &str, text: &str) -> PatternHintInput {
    PatternHintInput {
        node_kind: node_kind.to_string(),
        text: text.to_string(),
    }
}

// ── Acceptance criteria ───────────────────────────────────────────────────────

#[test]
fn expect_x_to_be_is_testing() {
    // Milestone acceptance criterion: `expect(x).toBe(1)` → `["testing"]`
    let result = classify_hint(&hint("call_expression", "expect(x).toBe(1)"), "javascript");
    assert_eq!(result, vec!["testing"]);
}

#[test]
fn describe_s_fn_is_testing() {
    // Milestone acceptance criterion: `describe('s', fn)` → `["testing"]`
    let result = classify_hint(&hint("call_expression", "describe('s', fn)"), "typescript");
    assert_eq!(result, vec!["testing"]);
}

// ── TypeScript / JavaScript ───────────────────────────────────────────────────

#[test]
fn it_does_is_testing() {
    let result = classify_hint(
        &hint("call_expression", "it('does something', fn)"),
        "javascript",
    );
    assert_eq!(result, vec!["testing"]);
}

#[test]
fn test_is_truthy_is_testing() {
    let result = classify_hint(
        &hint("call_expression", "test('is truthy', () => {})"),
        "typescript",
    );
    assert_eq!(result, vec!["testing"]);
}

#[test]
fn before_each_is_testing() {
    let result = classify_hint(
        &hint("call_expression", "beforeEach(() => {})"),
        "typescript",
    );
    assert_eq!(result, vec!["testing"]);
}

#[test]
fn after_all_is_testing() {
    let result = classify_hint(&hint("call_expression", "afterAll(() => {})"), "javascript");
    assert_eq!(result, vec!["testing"]);
}

#[test]
fn jest_mock_is_testing() {
    let result = classify_hint(
        &hint("call_expression", "jest.mock('./module')"),
        "typescript",
    );
    assert_eq!(result, vec!["testing"]);
}

#[test]
fn vi_fn_is_testing() {
    let result = classify_hint(&hint("call_expression", "vi.fn()"), "javascript");
    assert_eq!(result, vec!["testing"]);
}

// ── Go ────────────────────────────────────────────────────────────────────────

#[test]
fn t_run_is_testing_go() {
    let result = classify_hint(&hint("call_expression", "t.Run(\"sub\", fn)"), "go");
    assert_eq!(result, vec!["testing"]);
}

#[test]
fn t_fatal_is_testing_go() {
    let result = classify_hint(&hint("call_expression", "t.Fatal(err)"), "go");
    assert_eq!(result, vec!["testing"]);
}

// ── Python ────────────────────────────────────────────────────────────────────

#[test]
fn self_assert_equal_is_testing_python() {
    let result = classify_hint(&hint("call_expression", "self.assertEqual(a, b)"), "python");
    assert_eq!(result, vec!["testing"]);
}

#[test]
fn self_assert_true_is_testing_python() {
    let result = classify_hint(&hint("call_expression", "self.assertTrue(x)"), "python");
    assert_eq!(result, vec!["testing"]);
}

// ── Negatives — non-test calls stay in their category ────────────────────────

#[test]
fn console_log_is_logging_not_testing() {
    let result = classify_hint(&hint("call_expression", "console.log(x)"), "typescript");
    assert_ne!(result, vec!["testing"]);
    assert_eq!(result, vec!["logging"]);
}

#[test]
fn axios_get_is_data_access_not_testing() {
    let result = classify_hint(&hint("call_expression", "axios.get(url)"), "typescript");
    assert_eq!(result, vec!["data_access"]);
}

#[test]
fn use_effect_is_framework_hooks_not_testing() {
    let result = classify_hint(&hint("call_expression", "useEffect(fn, [])"), "typescript");
    assert_eq!(result, vec!["framework_hooks"]);
}

#[test]
fn self_method_is_not_testing() {
    // `self.method()` — no `assert[A-Z]` prefix; not matched
    let result = classify_hint(&hint("call_expression", "self.method()"), "python");
    assert!(
        !result.contains(&"testing"),
        "self.method() must not match testing; got {result:?}"
    );
}

// ── list_categories includes testing ─────────────────────────────────────────

#[test]
fn list_categories_includes_testing() {
    let catalog = sdivi_core::list_categories();
    let names: Vec<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.contains(&"testing"),
        "list_categories must include 'testing' (M42)"
    );
}

#[test]
fn list_categories_count_is_eighteen() {
    let catalog = sdivi_core::list_categories();
    assert_eq!(
        catalog.categories.len(),
        18,
        "list_categories must return exactly 18 categories after M44"
    );
}
