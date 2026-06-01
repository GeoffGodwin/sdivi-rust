//! M44 acceptance-criterion tests: `concurrency` pattern category.
//!
//! Verifies that:
//! - `category_for_node_kind("go_statement", "go") == Some("concurrency")`
//! - `category_for_node_kind("select_statement", "go") == Some("concurrency")`
//! - `classify_hint` routes `Promise.all/allSettled/race/any` (TS/JS) and
//!   `asyncio.gather/create_task/wait/as_completed/run` (Python) to `["concurrency"]`.
//! - Non-concurrency calls are not miscategorised.

use sdivi_patterns::queries::{category_for_node_kind, classify_hint};
use sdivi_patterns::PatternHintInput;

fn hint(node_kind: &str, text: &str) -> PatternHintInput {
    PatternHintInput {
        node_kind: node_kind.to_string(),
        text: text.to_string(),
    }
}

// ── M44 acceptance criteria ───────────────────────────────────────────────────

#[test]
fn go_statement_is_concurrency() {
    // Milestone acceptance criterion: category_for_node_kind("go_statement", "go") == Some("concurrency")
    assert_eq!(
        category_for_node_kind("go_statement", "go"),
        Some("concurrency"),
        "go_statement must map to concurrency (M44 acceptance criterion)"
    );
}

#[test]
fn select_statement_is_concurrency() {
    // Milestone acceptance criterion: category_for_node_kind("select_statement", "go") == Some("concurrency")
    assert_eq!(
        category_for_node_kind("select_statement", "go"),
        Some("concurrency"),
        "select_statement must map to concurrency (M44 acceptance criterion)"
    );
}

// ── classify_hint routing — Go node kinds ────────────────────────────────────

#[test]
fn classify_hint_go_statement_is_concurrency() {
    // go_statement is not a call_expression — it goes through category_for_node_kind via the `other` arm.
    let result = classify_hint(&hint("go_statement", "go worker(ch)"), "go");
    assert_eq!(result, vec!["concurrency"]);
}

#[test]
fn classify_hint_select_statement_is_concurrency() {
    let result = classify_hint(
        &hint("select_statement", "select { case msg := <-ch: }"),
        "go",
    );
    assert_eq!(result, vec!["concurrency"]);
}

// ── TypeScript / JavaScript ───────────────────────────────────────────────────

#[test]
fn promise_all_is_concurrency_ts() {
    let result = classify_hint(
        &hint("call_expression", "Promise.all([a, b])"),
        "typescript",
    );
    assert_eq!(result, vec!["concurrency"]);
}

#[test]
fn promise_all_settled_is_concurrency_js() {
    let result = classify_hint(
        &hint("call_expression", "Promise.allSettled([p1, p2])"),
        "javascript",
    );
    assert_eq!(result, vec!["concurrency"]);
}

#[test]
fn promise_race_is_concurrency_js() {
    let result = classify_hint(
        &hint("call_expression", "Promise.race([a, b])"),
        "javascript",
    );
    assert_eq!(result, vec!["concurrency"]);
}

#[test]
fn promise_any_is_concurrency_ts() {
    let result = classify_hint(
        &hint("call_expression", "Promise.any([a, b])"),
        "typescript",
    );
    assert_eq!(result, vec!["concurrency"]);
}

// ── Python ────────────────────────────────────────────────────────────────────

#[test]
fn asyncio_gather_is_concurrency_python() {
    let result = classify_hint(&hint("call", "asyncio.gather(*tasks)"), "python");
    assert_eq!(result, vec!["concurrency"]);
}

#[test]
fn asyncio_create_task_is_concurrency_python() {
    let result = classify_hint(&hint("call", "asyncio.create_task(coro())"), "python");
    assert_eq!(result, vec!["concurrency"]);
}

#[test]
fn asyncio_run_is_concurrency_python() {
    let result = classify_hint(&hint("call", "asyncio.run(main())"), "python");
    assert_eq!(result, vec!["concurrency"]);
}

// ── Negatives — boundary with async_patterns ─────────────────────────────────

#[test]
fn promise_then_is_async_patterns_not_concurrency() {
    let result = classify_hint(&hint("call_expression", "promise.then(r)"), "typescript");
    assert_eq!(result, vec!["async_patterns"]);
}

#[test]
fn promise_resolve_is_not_concurrency() {
    // Promise.resolve is not a multi-future coordination function
    let result = classify_hint(&hint("call_expression", "Promise.resolve(x)"), "javascript");
    assert!(
        !result.contains(&"concurrency"),
        "Promise.resolve must not be concurrency; got {result:?}"
    );
}

#[test]
fn defer_statement_is_not_concurrency() {
    // defer_statement belongs to resource_management (M45.1), not concurrency.
    let result = classify_hint(&hint("defer_statement", "defer f()"), "go");
    assert!(
        !result.contains(&"concurrency"),
        "defer_statement must not map to concurrency; got {result:?}"
    );
}

// ── list_categories contract ──────────────────────────────────────────────────

#[test]
fn list_categories_includes_concurrency() {
    let catalog = sdivi_core::list_categories();
    let names: Vec<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.contains(&"concurrency"),
        "list_categories must include 'concurrency' (M44)"
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
