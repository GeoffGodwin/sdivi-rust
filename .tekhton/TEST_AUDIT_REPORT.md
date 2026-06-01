## Test Audit Report

### Audit Summary
Tests audited: 5 files, 29 test functions
Verdict: **NEEDS_WORK**

---

### Findings

#### INTEGRITY: All four tests in test_null_safety_doc_clarity.rs assert against hardcoded strings they define themselves
- File: `crates/sdivi-patterns/tests/test_null_safety_doc_clarity.rs:8`, `:29`, `:50`, `:66`
- Issue: Every test in this file constructs a local string variable (`module_doc`, `node_kinds_doc`, `doc_example`, `doc`) whose body already contains the exact substring being asserted. For example, `null_safety_node_kinds_const_doc_clarifies_optional_calls` defines `node_kinds_doc` to include the text `"optional calls (`fn?.()`) emit `call_expression`"` and then asserts `node_kinds_doc.contains("optional calls (`fn?.()`) emit `call_expression`")`. The assertion is a tautology — it verifies only the string the tester wrote, not the actual source file `crates/sdivi-patterns/src/queries/null_safety.rs`. Confirmed by reading that file: the NODE_KINDS const doc at lines 31–49 contains the correct language, but the tests do not read or reference it. The actual source can be rewritten to remove the clarification entirely without any of these four tests failing.
- Severity: HIGH
- Action: Delete or rewrite all four tests. To actually verify doc content: read the source file using `include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/queries/null_safety.rs"))` and assert against that string, or rely on the inline doc tests already present in `null_safety.rs` (lines 44–49) plus the inline unit tests at lines 53–69. The tester-written file adds no coverage.

---

#### INTEGRITY: null_safety_doc_clarifies_optional_calls_emit_call_expression asserts against a self-contained string
- File: `crates/sdivi-patterns/tests/test_optional_chain_vs_call_expression.rs:53`
- Issue: The test defines a local variable `doc` whose body already contains the two asserted substrings (`"optional calls (`fn?.()`) emit `call_expression`"` and `"not `optional_chain`"`), then asserts those substrings are present. Always passes regardless of what the actual `null_safety.rs` source says. The other five tests in this file (`optional_chain_is_null_safety`, `non_null_expression_is_null_safety`, `optional_chain_node_kind_is_in_node_kinds`, `non_null_expression_node_kind_is_in_node_kinds`, `null_safety_node_kinds_list_correct_entries`) are legitimate — they call real `category_for_node_kind` and `NODE_KINDS.contains` against the actual implementation.
- Severity: HIGH
- Action: Remove only `null_safety_doc_clarifies_optional_calls_emit_call_expression`. Keep the five real-behavior tests.

---

#### INTEGRITY: concurrency_module_has_sql_adapter_seed_comment asserts against a self-contained string
- File: `crates/sdivi-patterns/tests/test_select_statement_sql_guard.rs:56`
- Issue: The test defines `seed_comment` as a string literal that already contains all six asserted substrings (`"SQL adapter seed"`, `"select_statement"`, `"SQL language adapter"`, `"PATTERN_KINDS"`, `"_language"`, `"per-language override"`), then asserts those substrings are present. The actual doc comment in `concurrency.rs` at lines 50–55 could be rewritten to remove any of these terms without this test failing. The five real-behavior tests in this file (`concurrency_node_kinds_contains_select_statement`, `concurrency_node_kinds_contains_go_statement`, `concurrency_node_kinds_list_is_complete`, `concurrency_matches_callee_promise_all_typescript`, `concurrency_matches_callee_asyncio_gather_python`) are legitimate and exercise the real `NODE_KINDS` constant and `matches_callee` function.
- Severity: HIGH
- Action: Remove `concurrency_module_has_sql_adapter_seed_comment`. Keep the five real-behavior tests. Also remove `concurrency_go_statement_and_select_statement_are_go_only` (see next finding).

---

#### INTEGRITY: concurrency_go_statement_and_select_statement_are_go_only asserts against a self-contained string
- File: `crates/sdivi-patterns/tests/test_select_statement_sql_guard.rs:79`
- Issue: Same pattern. The test defines a local `doc` string whose body contains `"Go adapter"` and `"sdivi-lang-go"`, then asserts those substrings are present. Always passes. The `concurrency::NODE_KINDS.contains` tests above already verify relevant runtime behavior.
- Severity: HIGH
- Action: Remove this test.

---

#### INTEGRITY: wasm_types_module_compiles_without_doc_warnings contains assert!(true)
- File: `bindings/sdivi-wasm/tests/test_wasm_doc_no_unresolved_links.rs:12`
- Issue: `assert!(true)` is a tautological assertion that can never fail under any circumstances. The comment claims this verifies compilation success, but a compilation test requires no assertion body — if the crate failed to compile, `cargo test` would not reach any test function. Three other tests in this file (`wasm_boundary_prior_partition_type_exists`, `wasm_boundary_inference_result_type_exists`, `wasm_trend_result_type_exists`) are legitimate compile-time type-existence tests.
- Severity: HIGH
- Action: Remove `wasm_types_module_compiles_without_doc_warnings`. The three type-instantiation tests below it already serve as implicit compilation checks.

---

#### INTEGRITY: wasm_types_doc_references_correct_functions asserts against a self-contained string
- File: `bindings/sdivi-wasm/tests/test_wasm_doc_no_unresolved_links.rs:20`
- Issue: Defines `doc_snippet` as a string literal containing `` `infer_boundaries` ``, `` `compute_trend` ``, and `sdivi_core::SnapshotPriorPartition`, then asserts those substrings are present. Read the actual `bindings/sdivi-wasm/src/types.rs` (lines 84–91 and 218–238): `WasmSnapshotPriorPartition` doc uses plain text (not intra-doc links), `WasmBoundaryInferenceResult` doc says "Output of `infer_boundaries`", and `WasmTrendResult` doc says "Output of `compute_trend`". The test does not read any of these — it asserts against its own hardcoded content.
- Severity: HIGH
- Action: Remove this test. The three type-instantiation tests confirm the types exist. Whether a doc *comment* uses specific wording is better enforced by `cargo doc --no-deps -D warnings` in CI (which the coder confirms is the actual gate) rather than a Rust unit test.

---

#### SCOPE: Shell-detected orphans are false positives — do not act on them
- File: All 5 test files listed in the orphan report
- Issue: The pre-verified orphan detection reports all five test files "import deleted module '.tekhton/.commit_decision'". Reading the actual content of each file confirms none contain any reference to `.tekhton`, `.commit_decision`, or any non-Rust path. `.tekhton/.commit_decision` is not a valid Rust module path. These are newly-created Rust test files (all listed as untracked `??` in git status at the start of this run). The detection script is generating false positives — likely matching against context metadata rather than the test file bodies. This is a recurring systemic issue (same false-positive class seen in prior audit runs).
- Severity: MEDIUM
- Action: Do NOT remove any test file on the basis of this orphan report. Fix or retire the orphan-detection script; it has produced false positives in consecutive audit cycles.

---

#### COVERAGE: callee_only_categories_listed_in_doc_match_real_dispatch is a trivially-true loop
- File: `crates/sdivi-patterns/tests/test_all_categories_doc_classification.rs:59`
- Issue: The test calls `category_for_node_kind("call_expression", "typescript")` eight times in a loop (once per callee-only category name) and uses `assert_ne!` to verify the result differs from each category name. In the actual implementation (`mod.rs:116`), `data_access::NODE_KINDS = &["call_expression", "call"]`, so this call always returns `Some("data_access")`. Every iteration reduces to `assert_ne!(Some("data_access"), Some("logging"))` etc. — trivially true for every callee-only name. The test would still pass even if `category_for_node_kind` returned `None` for `"call_expression"`. The companion test `callee_only_categories_have_empty_node_kinds()` (line 143) correctly tests the actual invariant (empty NODE_KINDS slice).
- Severity: MEDIUM
- Action: Remove `callee_only_categories_listed_in_doc_match_real_dispatch`. The `callee_only_categories_have_empty_node_kinds` test already verifies the meaningful property.

---

#### NAMING: All test names in test_null_safety_doc_clarity.rs imply source-file verification they do not perform
- File: `crates/sdivi-patterns/tests/test_null_safety_doc_clarity.rs:7`, `:28`, `:50`, `:66`
- Issue: Names such as `null_safety_module_doc_mentions_fn_optional_call_clarification` and `null_safety_node_kinds_const_doc_clarifies_optional_calls` suggest the tests read and check the actual source module doc. They do not — they assert against hardcoded strings (see HIGH INTEGRITY findings). Future readers may trust these names to guard doc regressions when they provide no such guard.
- Severity: LOW
- Action: Covered by the HIGH INTEGRITY action (remove all four tests). If rewritten using `include_str!`, update names to clarify the file-read approach (e.g., `null_safety_source_doc_mentions_optional_call_clarification`).
