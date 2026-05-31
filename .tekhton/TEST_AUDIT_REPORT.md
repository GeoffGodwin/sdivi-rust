## Test Audit Report

### Audit Summary
Tests audited: 2 files, 36 test functions
- `crates/sdivi-lang-typescript/tests/extract_behavior.rs`: 22 test functions
- `crates/sdivi-lang-javascript/tests/extract_behavior.rs`: 14 test functions

Verdict: PASS

---

### Findings

#### SCOPE: Pre-verified orphan claims are false positives
- File: crates/sdivi-lang-typescript/tests/extract_behavior.rs (entire file)
- File: crates/sdivi-lang-javascript/tests/extract_behavior.rs (entire file)
- Issue: The audit context lists four "ORPHAN" entries asserting both test files "import deleted module '.tekhton/.commit_decision'". Direct grep of both files finds zero occurrences of "commit_decision" or ".tekhton". The claims are false positives. Both files' only imports are `sdivi_lang_typescript`/`sdivi_lang_javascript`, `sdivi_parsing::adapter::LanguageAdapter`, `sdivi_parsing::feature_record::FeatureRecord`, and `std::path::Path` — all of which resolve to live, unmodified modules. The orphan-detection script appears to have scanned a stale artifact or pattern-matched against something outside the source files.
- Severity: LOW
- Action: No action needed in the test files themselves. Investigate the orphan detection script for the false-positive source; do not remove or modify either test file on the basis of these claims.

#### COVERAGE: JS decorator positive path is vacuously passing
- File: crates/sdivi-lang-javascript/tests/extract_behavior.rs:188–214
- Issue: `decorator_hint_is_correctly_typed_if_grammar_emits_it` tests correctness of decorator hints *if* the tree-sitter-javascript grammar emits them. If the grammar does not emit `decorator` nodes for the `@injectable\nclass Service {}` input, `decorator_hints` is empty, the `for` loop body never executes, and the test passes with zero assertions evaluated. This is acknowledged in the comment, but the consequence is that the JS positive decorator code path — `"decorator"` in `PATTERN_KINDS` → collected hint — is never concretely exercised by any test in this file. Only the negative path (`file_with_no_decorators_produces_no_decorator_hints`) produces a concrete assertion.
- Severity: MEDIUM
- Action: Determine whether the current tree-sitter-javascript grammar version emits `decorator` nodes for Stage-3 syntax (add `eprintln!("{:?}", record.pattern_hints)` to inspect CI output). If it does: add `assert!(!decorator_hints.is_empty(), "grammar must emit decorator nodes for @injectable syntax")` before the loop. If it does not: document that fact explicitly in the test comment and remove the dead loop body, then note in `MIGRATION_NOTES.md` that JS decorator collection is inert until the grammar gains support. Either way, replace the vacuous guard with a concrete, factual statement about the grammar's behavior.

#### INTEGRITY: None
No tests assert hard-coded magic values unconnected to implementation logic. All assertions derive from real `parse_file()` calls on known source strings. The count `>= 3` in `nestjs_shaped_controller_yields_multiple_decorator_hints` is grounded in the fixture (3 explicit decorators: `@Controller`, `@Get`, `@Post`). The `starts_with('@')` assertion in `decorator_hint_text_starts_with_at_sign` reflects the actual text tree-sitter extracts for `decorator` nodes. No tautologies found.

#### WEAKENING: None
Both test files were modified by appending a new `// ── decorator pattern hints (M36.1) ──` section. Pre-existing tests (import extraction, export extraction, `try_statement`, `await_expression`, byte-truncation, M31 class hierarchy) are present and byte-identical to their prior state. No existing assertion was broadened, narrowed, or removed.

#### ISOLATION: None
All tests construct source code as inline string literals and call `parse_file()` directly. No test reads from `.tekhton/` reports, snapshot directories, build logs, `Cargo.lock`, or any mutable project state. Fixture isolation is sound throughout both files.

#### NAMING: None
All new test function names encode the scenario and expected outcome. Representative examples: `nestjs_shaped_controller_yields_multiple_decorator_hints`, `file_with_no_decorators_produces_no_decorator_hints`, `decorator_hint_text_starts_with_at_sign`, `decorator_hint_is_correctly_typed_if_grammar_emits_it`. No opaque names found.

#### EXERCISE: None
Both test files call `TypeScriptAdapter.parse_file()` and `JavaScriptAdapter.parse_file()` — real tree-sitter parsing with no mocking. Assertions verify `node_kind`, `text`, and counts on fields of the returned `FeatureRecord`. The implementation path from `PATTERN_KINDS.contains(&node.kind())` through `collect_hints` → `PatternHint` construction is exercised by every decorator test.

---

### Rubric Scorecard

| # | Criterion | Verdict | Notes |
|---|---|---|---|
| 1 | Assertion Honesty | PASS | All assertions are derived from real function calls. No always-passing assertions found. |
| 2 | Edge Case Coverage | PASS (with note) | Negative path tested in both files. JS positive path is vacuously guarded — MEDIUM gap noted above. |
| 3 | Implementation Exercise | PASS | Real adapters called; no mocks at any level. |
| 4 | Test Weakening | PASS | Tester changes are purely additive. No existing assertion modified. |
| 5 | Naming and Intent | PASS | All 36 test names encode scenario + expected outcome. |
| 6 | Scope Alignment | PASS | Orphan claims are false positives confirmed by grep. No deleted or renamed APIs referenced. |
| 7 | Test Isolation | PASS | All tests use inline source strings. No external file reads. |

---

### Tester Claim Verification

**Claim: `extract_behavior.rs` (TS) — 6 new decorator tests via real tree-sitter parse (NestJS-shaped fixture).**
Confirmed. Tests at lines 237–357: `class_decorator_captured_as_pattern_hint`, `method_decorator_captured_as_pattern_hint`, `nestjs_shaped_controller_yields_multiple_decorator_hints`, `decorator_hint_text_starts_with_at_sign`, `multiple_class_decorators_each_produce_a_hint`, `file_with_no_decorators_produces_no_decorator_hints`. All call `parse_ts()` against inline fixtures and assert against real `FeatureRecord` output. ✓

**Claim: `extract_behavior.rs` (JS) — JS decorator hint test (grammar-dependent; documents behavior).**
Confirmed with caveat. Tests at lines 172–214: `file_with_no_decorators_produces_no_decorator_hints` is concrete and passes unconditionally. `decorator_hint_is_correctly_typed_if_grammar_emits_it` is intentionally permissive (grammar-dependent) but produces no concrete assertion if the grammar does not emit decorator nodes. See COVERAGE finding above.

**Claim: Passed: 46 Failed: 0.**
Cannot independently verify (no test run performed). The test logic is internally consistent with the implementation for all TS decorator tests. The JS decorator positive test is vacuous if the grammar does not emit decorator nodes, which would inflate the pass count without exercising the code path.
