//! Tests for `framework_hooks` pattern category (M35).
//!
//! Verifies `classify_hint` dispatch and `framework_hooks::matches_callee` in
//! isolation: positive cases (built-in and custom hooks), negative cases
//! (lowercase second char, non-hook names, wrong languages).

use sdivi_patterns::queries::{classify_hint, framework_hooks};
use sdivi_patterns::PatternHintInput;

fn hint(node_kind: &str, text: &str) -> PatternHintInput {
    PatternHintInput {
        node_kind: node_kind.to_string(),
        text: text.to_string(),
    }
}

// ── classify_hint acceptance criteria ─────────────────────────────────────────

#[test]
fn classify_hint_use_state_is_framework_hooks() {
    assert_eq!(
        classify_hint(&hint("call_expression", "useState(0)"), "typescript"),
        vec!["framework_hooks"]
    );
}

#[test]
fn classify_hint_use_effect_is_framework_hooks() {
    assert_eq!(
        classify_hint(&hint("call_expression", "useEffect(fn, [])"), "typescript"),
        vec!["framework_hooks"]
    );
}

#[test]
fn classify_hint_use_memo_is_framework_hooks() {
    assert_eq!(
        classify_hint(
            &hint("call_expression", "useMemo(() => val, [])"),
            "javascript"
        ),
        vec!["framework_hooks"]
    );
}

#[test]
fn classify_hint_custom_hook_is_framework_hooks() {
    assert_eq!(
        classify_hint(
            &hint("call_expression", "useCustomHook(opts)"),
            "typescript"
        ),
        vec!["framework_hooks"]
    );
    assert_eq!(
        classify_hint(&hint("call_expression", "useAuth()"), "javascript"),
        vec!["framework_hooks"]
    );
}

#[test]
fn classify_hint_call_node_kind_also_dispatches() {
    assert_eq!(
        classify_hint(&hint("call", "useState(0)"), "typescript"),
        vec!["framework_hooks"]
    );
}

#[test]
fn classify_hint_user_lowercase_returns_empty() {
    assert_eq!(
        classify_hint(&hint("call_expression", "user()"), "typescript"),
        Vec::<&str>::new()
    );
}

#[test]
fn classify_hint_get_user_returns_empty() {
    assert_eq!(
        classify_hint(&hint("call_expression", "getUser()"), "typescript"),
        Vec::<&str>::new()
    );
}

// ── matches_callee positive cases ─────────────────────────────────────────────

#[test]
fn matches_callee_built_in_hooks_typescript() {
    let hooks = [
        "useState(0)",
        "useEffect(fn, [])",
        "useMemo(() => v, [])",
        "useCallback(fn, [])",
        "useRef(null)",
        "useContext(Ctx)",
        "useReducer(reducer, s)",
        "useLayoutEffect(fn, [])",
    ];
    for callee in hooks {
        assert!(
            framework_hooks::matches_callee(callee, "typescript"),
            "{callee:?} should match for typescript"
        );
    }
}

#[test]
fn matches_callee_custom_hooks_javascript() {
    assert!(framework_hooks::matches_callee("useAuth()", "javascript"));
    assert!(framework_hooks::matches_callee("useStore()", "javascript"));
    assert!(framework_hooks::matches_callee(
        "useMutation(fn)",
        "javascript"
    ));
    assert!(framework_hooks::matches_callee(
        "useQuery(key)",
        "javascript"
    ));
}

// ── matches_callee negative cases ─────────────────────────────────────────────

#[test]
fn matches_callee_lowercase_second_char_no_match() {
    // Must NOT match: `use` followed by a lowercase letter
    assert!(!framework_hooks::matches_callee("user()", "typescript"));
    assert!(!framework_hooks::matches_callee("username()", "typescript"));
    assert!(!framework_hooks::matches_callee("useful(x)", "javascript"));
    assert!(!framework_hooks::matches_callee("fuse(x)", "typescript"));
    // `используем` (Cyrillic) — mid-identifier, also no match
    assert!(!framework_hooks::matches_callee(
        "используем(x)",
        "typescript"
    ));
}

#[test]
fn matches_callee_non_use_prefix_no_match() {
    assert!(!framework_hooks::matches_callee("getUser()", "typescript"));
    assert!(!framework_hooks::matches_callee(
        "setState(x)",
        "typescript"
    ));
    assert!(!framework_hooks::matches_callee(
        "Math.max(a, b)",
        "typescript"
    ));
    assert!(!framework_hooks::matches_callee(
        "console.log(x)",
        "typescript"
    ));
    assert!(!framework_hooks::matches_callee("fetch(url)", "typescript"));
}

#[test]
fn matches_callee_wrong_language_no_match() {
    for lang in ["rust", "python", "go", "java"] {
        assert!(
            !framework_hooks::matches_callee("useState(0)", lang),
            "useState should not match for language={lang:?}"
        );
        assert!(
            !framework_hooks::matches_callee("useEffect(fn)", lang),
            "useEffect should not match for language={lang:?}"
        );
    }
}
