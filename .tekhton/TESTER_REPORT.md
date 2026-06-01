## Planned Tests
- [x] `crates/sdivi-patterns/tests/test_all_categories_doc_classification.rs` — verify ALL_CATEGORIES doc correctly lists hybrid vs node-kind-only categories
- [x] `crates/sdivi-patterns/tests/test_optional_chain_vs_call_expression.rs` — verify optional calls emit call_expression not optional_chain
- [x] `crates/sdivi-patterns/tests/test_null_safety_doc_clarity.rs` — verify null_safety docs correctly note fn?.() emits call_expression
- [x] `crates/sdivi-patterns/tests/test_select_statement_sql_guard.rs` — verify concurrency.rs has SQL adapter warning comment
- [x] `bindings/sdivi-wasm/tests/test_wasm_doc_no_unresolved_links.rs` — verify resolved rustdoc links in types.rs

## Test Run Results
Passed: 29  Failed: 0

## Bugs Found
None

## Files Modified
- [x] `crates/sdivi-patterns/tests/test_all_categories_doc_classification.rs`
- [x] `crates/sdivi-patterns/tests/test_optional_chain_vs_call_expression.rs`
- [x] `crates/sdivi-patterns/tests/test_null_safety_doc_clarity.rs`
- [x] `crates/sdivi-patterns/tests/test_select_statement_sql_guard.rs`
- [x] `bindings/sdivi-wasm/tests/test_wasm_doc_no_unresolved_links.rs`
