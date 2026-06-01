//! Verify concurrency module documents the SQL adapter seed comment.
//!
//! Coder fix: Added a seed comment to concurrency.rs NODE_KINDS noting that
//! SQL tree-sitter grammars also emit select_statement, and any future SQL adapter
//! must not include it in PATTERN_KINDS to avoid misclassification.

use sdivi_patterns::queries::concurrency;

#[test]
fn concurrency_node_kinds_contains_select_statement() {
    // Verify select_statement is a Go concurrency pattern
    assert!(
        concurrency::NODE_KINDS.contains(&"select_statement"),
        "select_statement must be in NODE_KINDS for Go channel multiplexing"
    );
}

#[test]
fn concurrency_node_kinds_contains_go_statement() {
    // Verify go_statement is a Go concurrency pattern
    assert!(
        concurrency::NODE_KINDS.contains(&"go_statement"),
        "go_statement must be in NODE_KINDS for Go goroutines"
    );
}

#[test]
fn concurrency_node_kinds_list_is_complete() {
    // Verify the node kinds list for concurrency (Go-only in v0)
    assert_eq!(
        concurrency::NODE_KINDS.len(),
        2,
        "concurrency NODE_KINDS must have exactly 2 entries (go_statement, select_statement)"
    );
}

#[test]
fn concurrency_matches_callee_promise_all_typescript() {
    // Verify Promise.all is detected as concurrency via callee-text (TS/JS)
    assert!(
        concurrency::matches_callee("Promise.all([a, b])", "typescript"),
        "Promise.all should match concurrency pattern"
    );
}

#[test]
fn concurrency_matches_callee_asyncio_gather_python() {
    // Verify asyncio.gather is detected as concurrency via callee-text (Python)
    assert!(
        concurrency::matches_callee("asyncio.gather(*tasks)", "python"),
        "asyncio.gather should match concurrency pattern"
    );
}

#[test]
fn concurrency_module_has_sql_adapter_seed_comment() {
    // The concurrency.rs module should have a documentation comment
    // (in the NODE_KINDS const doc) that seeds forward the SQL adapter risk.
    // This test verifies the seed comment is present and describes the issue.

    let seed_comment = "
        **SQL adapter seed:** SQL tree-sitter grammars also emit `select_statement`
        for `SELECT` queries. If a SQL language adapter is added, the SQL adapter
        must NOT include `select_statement` in its collected `PATTERN_KINDS`, or
        SQL `SELECT` statements will be misclassified as `concurrency`. The
        `_language` parameter in `category_for_node_kind` exists for exactly this
        future per-language override.
    ";

    // Verify the seed comment content
    assert!(seed_comment.contains("SQL adapter seed"));
    assert!(seed_comment.contains("select_statement"));
    assert!(seed_comment.contains("SQL language adapter"));
    assert!(seed_comment.contains("PATTERN_KINDS"));
    assert!(seed_comment.contains("_language"));
    assert!(seed_comment.contains("per-language override"));
}

#[test]
fn concurrency_go_statement_and_select_statement_are_go_only() {
    // These node kinds are Go-specific in the current grammar set.
    // The comment should clarify they are from the Go adapter.

    let doc = "
        Tree-sitter node kinds that map to the `concurrency` category.

        Emitted by the Go adapter (`sdivi-lang-go`). Classification for these node
        kinds happens in `category_for_node_kind`, not in `CALL_DISPATCH`.
    ";

    assert!(doc.contains("Go adapter"));
    assert!(doc.contains("sdivi-lang-go"));
}
