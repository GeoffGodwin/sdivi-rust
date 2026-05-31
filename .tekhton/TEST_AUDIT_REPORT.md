## Test Audit Report

### Audit Summary
Tests audited: 4 files, 34 test functions
- `crates/sdivi-patterns/tests/dispatch_disjointness.rs`: 4 test functions
- `crates/sdivi-patterns/tests/framework_hooks.rs`: 12 test functions
- `crates/sdivi-patterns/src/queries/framework_hooks.rs` (inline `#[cfg(test)] mod tests`): 6 test functions
- `crates/sdivi-core/tests/category_contract.rs`: 8 test functions (+4 internal mod tests in `mod.rs` observed, not audited)

Verdict: PASS

---

### Findings

#### SCOPE: Pre-verified orphan claims are false positives
- File: crates/sdivi-patterns/tests/dispatch_disjointness.rs, crates/sdivi-patterns/tests/framework_hooks.rs, crates/sdivi-patterns/src/queries/framework_hooks.rs, crates/sdivi-core/tests/category_contract.rs
- Issue: The audit context lists six "ORPHAN" entries asserting each file "imports deleted module '.tekhton/.commit_decision'". A direct grep of all four files finds zero occurrences of either "commit_decision" or ".tekhton". The claims are false positives. All actual imports in these files resolve to live modules: `sdivi_patterns::queries::{async_patterns, classify_hint, data_access, framework_hooks, logging}`, `sdivi_patterns::PatternHintInput`, `sdivi_core`, and `std::{collections::HashSet, fs, path::Path}`. The orphan-detection tool appears to have scanned a stale artifact set or produced incorrect output.
- Severity: MEDIUM
- Action: Investigate the orphan-detection tooling for the false-positive source. Do not remove or modify any of the four test files on the basis of these claims — no orphan exists in any of them.

#### COVERAGE: External framework_hooks test file duplicates inline tests without adding `matches_callee` boundary cases
- File: crates/sdivi-patterns/tests/framework_hooks.rs:88–167 vs crates/sdivi-patterns/src/queries/framework_hooks.rs:57–119
- Issue: `matches_callee_built_in_hooks_typescript`, `matches_callee_lowercase_second_char_no_match`, `matches_callee_non_use_prefix_no_match`, and `matches_callee_wrong_language_no_match` in the external file test the same inputs as `built_in_hooks_match_typescript`, `lowercase_second_char_does_not_match`, `non_use_prefix_does_not_match`, and `other_languages_return_false` in the inline tests. The external file adds genuine value only through the `classify_hint`-level tests (lines 19–83) and the `call` node kind test (line 62–67), which the inline tests cannot reach. The raw `matches_callee` duplication is harmless but adds no incremental coverage. Neither the inline nor external set tests the boundary inputs `"useA()"` (minimal valid hook — `use` + one uppercase letter) or `"use("` (bare prefix, no uppercase), which are implied by the regex `^use[A-Z]` but not explicitly pinned.
- Severity: LOW
- Action: Optionally add `assert!(framework_hooks::matches_callee("useA()", "typescript"))` and `assert!(!framework_hooks::matches_callee("use()", "typescript"))` to the boundary negative test. Not required for PASS.

#### ISOLATION: category_contract.rs reads live source and documentation files
- File: crates/sdivi-core/tests/category_contract.rs:178–225
- Issue: `no_category_string_in_patterns_src_missing_from_list_categories` recursively reads `crates/sdivi-patterns/src/` from the filesystem at test runtime using `CARGO_MANIFEST_DIR`. `markdown_table_matches_list_categories_output` reads `docs/pattern-categories.md`. These are stable, version-controlled source files — not build artifacts or pipeline state — so the isolation concern is lower than the rubric's illustrative examples (`.tekhton/*`, `.claude/logs/*`). However, the tests' pass/fail outcome can differ between a working tree with uncommitted edits and a clean checkout. If a developer edits `docs/pattern-categories.md` without running tests, a colleague's CI may report a failure they cannot reproduce.
- Severity: LOW
- Action: Acceptable as intentional drift-gate design. No change required. Consider adding a module-level comment in `category_contract.rs` explicitly noting that these tests are drift gates that intentionally read source-tree files, to prevent a future auditor from flagging the coupling as accidental.

#### COVERAGE: `corpus_resolves_identically_for_call_node_kind` delegates correctness silently
- File: crates/sdivi-patterns/tests/dispatch_disjointness.rs:143–153
- Issue: The test uses `_expected` and only asserts that the `call` and `call_expression` paths agree with each other — it does not independently verify the result is correct. This is valid delegation to `corpus_resolves_to_expected_category`, but if that test is later removed or the corpus shrinks, this test continues passing while providing no correctness signal. The delegation intent is not documented inside the test body.
- Severity: LOW
- Action: Add a one-line comment inside the test body: `// Correctness of results is verified by corpus_resolves_to_expected_category; this test only asserts routing parity between node kinds.`

---

### Rubric Scorecard

| # | Criterion | Verdict | Notes |
|---|---|---|---|
| 1 | Assertion Honesty | PASS | All assertions derive from real function calls. The count `9` in `list_categories_returns_exactly_nine_categories` is verified against the actual `CATALOG_ENTRIES` array (9 entries confirmed in `categories.rs:17–81`). No tautologies or always-passing assertions found across any file. |
| 2 | Edge Case Coverage | PASS | Negative paths covered across all three test files: empty-result corpus entries, wrong-language rejections, lowercase-second-char rejections, non-`use` prefix rejections, undocumented-overlap enforcement, loser category verification. Minor gap: boundary inputs `useA()` / `use()` not explicitly pinned (LOW). |
| 3 | Implementation Exercise | PASS | All tests call real implementations with no mocking. `classify_hint`, `framework_hooks::matches_callee`, `async_patterns::matches_callee`, `logging::matches_callee`, `data_access::matches_callee`, `sdivi_core::list_categories`, and `sdivi_core::CATEGORIES` are all exercised on real types. |
| 4 | Test Weakening | PASS | Tester changes to `dispatch_disjointness.rs` are additive only: new import, 7 CORPUS entries, 1 `all_matching_categories` branch, 1 `loser_matches` arm. The count update in `category_contract.rs` (8→9) correctly tracks the new entry. No existing assertion was broadened or removed across any file. |
| 5 | Naming and Intent | PASS | All 34 test function names encode the scenario and expected outcome (`classify_hint_use_state_is_framework_hooks`, `matches_callee_wrong_language_no_match`, `list_categories_returns_exactly_nine_categories`, `no_undocumented_overlaps_in_corpus`, etc.). No opaque names found. |
| 6 | Scope Alignment | PASS | All imports resolve to currently-implemented symbols. The pre-verified orphan claims are false positives confirmed by grep (see SCOPE finding above). No deleted or renamed APIs are referenced in any audit file. |
| 7 | Test Isolation | PASS (with note) | `dispatch_disjointness.rs` and `framework_hooks.rs` tests use only const tables and real function calls — no filesystem I/O, no `.tekhton/` reads, no pipeline state dependency. `category_contract.rs` reads source-tree files intentionally as a drift gate (see ISOLATION finding — LOW, not a blocking concern). |

---

### Tester Claim Verification

**Claim: `dispatch_disjointness.rs` — TODO comment added and P8>P9 overlap documented.**
Confirmed. TODO comment at lines 25–27 references M39 (next slot). Second `KNOWN_OVERLAPS` entry at lines 62–67 documents `logger.get("x")` / typescript with `logging` winning over `data_access`. The overlap is factually correct: `logging::matches_callee` matches `^(console|logger|log)\.` → true; `data_access::matches_callee` matches `\b(get)\(` (word boundary satisfied by the preceding `.`) → true; `classify_hint` returns `["logging"]` because logging (P8) precedes data_access (P9) in `CALL_DISPATCH`. ✓

**Claim: `framework_hooks.rs` — `classify_hint` and `matches_callee` acceptance criteria added.**
Confirmed. 12 test functions covering built-in and custom hooks, both node kind spellings, and four negative-case categories. All assert real behavior against real implementations. ✓

**Claim: `category_contract.rs` — count=9 and `framework_hooks` inclusion assertions added.**
Confirmed. `list_categories_returns_exactly_nine_categories` (line 132) and `list_categories_includes_framework_hooks` (line 142). Both assertions match the current `CATALOG_ENTRIES` array which contains exactly 9 entries including `framework_hooks` at index 4. ✓
