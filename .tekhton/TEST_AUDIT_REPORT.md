## Test Audit Report

### Audit Summary
Tests audited: 2 files, 10 test functions
Verdict: PASS

---

### Findings

#### SCOPE: Shell orphan-detection false positives on both test files
- File: `crates/sdivi-detection/tests/renumber_delegation.rs` (flagged as orphan)
- File: `crates/sdivi-patterns/tests/go_concurrency_node_kind.rs` (flagged as orphan)
- Issue: The pre-verified orphan list claims both files "import deleted module
  '.tekhton/.commit_decision'". Neither file references that path in any form — both
  were read in full and contain only valid Rust `use` declarations (`sdivi_detection::internal`,
  `sdivi_detection::partition`, `sdivi_patterns::queries`, `rand`). `.tekhton/.commit_decision`
  is not a Rust module path and cannot be imported by any Rust source file. This is a
  false positive in the orphan-detection script, not a test defect.
- Severity: LOW
- Action: Dismiss the orphan flags. No test changes needed. Investigate why the
  orphan-detection script produced these lines — likely a string-search false match on
  the filename rather than an actual Rust `use` analysis.

---

#### COVERAGE: Hardcoded node-kind list in `all_concurrency_node_kinds_are_classified`
- File: `crates/sdivi-patterns/tests/go_concurrency_node_kind.rs:68`
- Issue: The test iterates over a hardcoded `vec!["go_statement", "select_statement"]`
  instead of using `concurrency::NODE_KINDS` directly. Currently the list matches
  `concurrency::NODE_KINDS` exactly (confirmed: `&["go_statement", "select_statement"]`),
  so the test passes. If a third node kind is added to `NODE_KINDS` in a future milestone,
  this test will not automatically cover it — the gap will be silent.
- Severity: MEDIUM
- Action: Replace the local vec with `sdivi_patterns::queries::concurrency::NODE_KINDS`
  so the test always covers the canonical source-of-truth list:
  `for node_kind in sdivi_patterns::queries::concurrency::NODE_KINDS { ... }`.

---

#### INTEGRITY: Trivially-true `assert_ne!` calls against callee-only categories
- File: `crates/sdivi-patterns/tests/go_concurrency_node_kind.rs:88`
- Issue: `go_statement_not_misclassified` includes eight `assert_ne!` calls for
  callee-text-only categories: `collection_pipelines`, `framework_hooks`, `http_routing`,
  `logging`, `schema_validation`, `serialization`, `state_store`, `testing`. None of these
  categories appear in the `category_for_node_kind` dispatch in `queries/mod.rs` — the
  function does not check them and can never return them for any input. All eight
  assertions always pass regardless of implementation correctness. The core assertion at
  line 112 (`assert_eq!(result, Some("concurrency"))`) is real and catches regressions;
  the trivially-true `assert_ne!` calls inflate apparent coverage without adding safety.
- Severity: LOW
- Action: Remove the eight trivially-true `assert_ne!` assertions for callee-only
  categories, or add an inline comment making clear they document the API contract
  (`category_for_node_kind` does not dispatch callee-only categories) rather than
  serving as behavioral guards.

---

### Per-File Notes

#### `crates/sdivi-detection/tests/renumber_delegation.rs`

All four tests call the real `refine_partition` through the `sdivi_detection::internal`
re-export (confirmed present in `lib.rs:38`). `LeidenGraph::from_edges` exists
(`graph.rs:79`). `QualityFunction` is public via `sdivi_detection::partition`.

Assertions verify the density property `[0, k)` without hardcoding `k` — correct, since
the partition output depends on graph topology and seed. The determinism test
(`refine_partition_renumbering_deterministic_with_seed`) directly exercises the
seeded-RNG contract. All four tests exercise the `renumber_in_place → super::renumber`
delegation as the mechanism that produces dense IDs: a regression in the delegation
(e.g., reverting to a broken body) would cause at least tests 1, 2, and 4 to fail.

No orphaned imports, no hard-coded magic values, no mutable project-file reads, no
test weakening.

#### `crates/sdivi-patterns/tests/go_concurrency_node_kind.rs`

`category_for_node_kind("go_statement", "go") = Some("concurrency")` confirmed against
`concurrency::NODE_KINDS = &["go_statement", "select_statement"]` and the dispatch in
`queries/mod.rs:119`. The `defer_statement → resource_management` assertion (line 60)
confirmed against `resource_management::NODE_KINDS` (grep: line 23 of that file). The
language-parameter-ignored test is consistent with the `_language` prefix in the
implementation signature. Core assertions are grounded in implementation logic.

See COVERAGE and INTEGRITY findings above for the two improvement opportunities.
