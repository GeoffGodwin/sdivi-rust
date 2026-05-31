## Test Audit Report

### Audit Summary
Tests audited: 5 files, ~80 test functions (M36.2 Python decorator extension)
- `crates/sdivi-patterns/src/queries/decorators.rs`: 3 inline tests (1 new, 1 renamed)
- `crates/sdivi-lang-python/tests/extract_behavior.rs`: 29 tests (4 new in M36.2)
- `crates/sdivi-core/tests/category_contract.rs`: 11 tests (1 new in M36.2)
- `crates/sdivi-lang-typescript/tests/extract_behavior.rs`: 22 tests (0 new in M36.2)
- `crates/sdivi-lang-javascript/tests/extract_behavior.rs`: 15 tests (0 new in M36.2)

Verdict: CONCERNS

---

### Findings

#### SCOPE: Shell-detected orphan report is entirely false for all 5 files
- File: All 5 files listed in the "Shell-Detected Orphans (pre-verified)" block
- Issue: The pre-verified orphan report claims every audited file "imports deleted module '.tekhton/.commit_decision'". A workspace-wide grep (`grep -r "commit_decision" crates/ --include="*.rs"`) returns zero matches. After reading every file in full, none contains any `use`, `mod`, string literal, or comment referencing `.tekhton/.commit_decision`. The orphan detection script is producing entirely spurious results for all 5 listed files. This is an escalation from the M36.1 audit which flagged the same script as LOW for 2 files; M36.2 now shows the failure affects the full audit scope. If a downstream step removes tests on the basis of this report, five valid, passing test files would be destroyed.
- Severity: HIGH
- Action: Do not remove or modify any test file based on this orphan report. Investigate the orphan detection script — it is pattern-matching against something other than actual source imports. The underlying tests are correct and must be preserved.

#### SCOPE: Tester claims modification of files unchanged in M36.2
- File: `crates/sdivi-lang-typescript/tests/extract_behavior.rs`, `crates/sdivi-lang-javascript/tests/extract_behavior.rs`
- Issue: The tester's report lists both TS and JS files under "Files Modified." The git status snapshot shows neither file as modified in the working tree for this run. The last commit touching these files is `101bb68 [MILESTONE 36.1 ✓] feat: Pattern Category — decorators (TS/JS)`. The M36.2 coder summary makes no mention of changes to either file. The NestJS controller fixture and JS grammar-conditional tests that the tester describes as new work are pre-existing M36.1 content. The tester re-ran existing tests and listed the files as "modified," inflating the apparent scope of M36.2 testing.
- Severity: MEDIUM
- Action: The tester should distinguish "verified" (ran pre-existing tests, confirmed still pass — which is legitimate regression coverage) from "modified" (added or changed test logic). No action needed on the test code itself; the tests are correct.

#### COVERAGE: JS decorator positive path remains vacuously guarded (pre-existing, carried forward)
- File: `crates/sdivi-lang-javascript/tests/extract_behavior.rs:188`
- Issue: `decorator_hint_is_correctly_typed_if_grammar_emits_it` makes zero concrete assertions if the tree-sitter-javascript grammar does not emit `decorator` nodes. The loop body never executes and the test passes. This was flagged as MEDIUM in the M36.1 audit and has not been addressed. The M36.2 run did not modify this file, so no regression occurred, but the gap persists.
- Severity: MEDIUM
- Action: Determine whether the tree-sitter-javascript grammar emits `decorator` nodes for Stage-3 syntax. If it does: add `assert!(!decorator_hints.is_empty())` before the loop. If it does not: remove the dead loop body and document the grammar limitation explicitly. See M36.1 audit for full remediation detail.

#### INTEGRITY: None
No tests assert hard-coded magic values unconnected to implementation logic. All new M36.2 assertions derive from real `PythonAdapter.parse_file()` and `category_for_node_kind()` calls. The expected counts (2 for two decorated functions, 1 for stacked decorators, 0 for no decorators) are grounded in tree-sitter-python's `decorated_definition` wrapper semantics. No tautologies or always-passing assertions found.

#### WEAKENING: None
The rename of `node_kinds_has_exactly_one_entry` → `node_kinds_has_two_entries` in `decorators.rs` is a correct update, not a weakening — `NODE_KINDS.len()` is now asserted as 2 to match the extended constant. No existing assertion was broadened or removed.

#### ISOLATION: None (new tests)
All four new Python tests and the new `category_contract.rs` test construct fixtures as inline string literals or call pure functions. No new test reads from `.tekhton/` reports, snapshot directories, build logs, or any mutable project state. The pre-existing `markdown_table_matches_list_categories_output` test reads `docs/pattern-categories.md` — a checked-in documentation file, not a build artifact — which is appropriate for a doc/runtime parity gate.

#### NAMING: None
All new M36.2 test names encode both scenario and expected outcome: `decorated_definition_captured_as_decorator_hint`, `fastapi_and_pytest_fixture_produce_decorated_definition_hints`, `stacked_decorators_count_as_one_decorated_definition`, `file_with_no_decorators_produces_no_decorated_definition_hints`, `decorated_definition_python_is_decorators`. No opaque names found.

---

### Per-File Assessment

**`crates/sdivi-patterns/src/queries/decorators.rs`**
Three inline tests assert directly against the `NODE_KINDS` constant defined in the same file (`&["decorator", "decorated_definition"]`). All three are honest — they would fail if either entry were removed. The rename from `node_kinds_has_exactly_one_entry` → `node_kinds_has_two_entries` with the assertion updated from `1` to `2` is correct and not a weakening. **PASS.**

**`crates/sdivi-lang-python/tests/extract_behavior.rs`**
Four new M36.2 tests are thorough: happy path (`@dataclass` class → one `decorated_definition` hint), integration scenario (two decorated functions → two hints), wrapper-granularity semantics (three stacked `@`-lines on one function → one hint, documenting the Python count asymmetry vs. TS/JS), and negative case (no decorators → zero hints). All call `PythonAdapter.parse_file()` on real source strings with no mocking. Edge cases present and meaningful. **PASS.**

**`crates/sdivi-core/tests/category_contract.rs`**
New test `decorated_definition_python_is_decorators` calls `category_for_node_kind("decorated_definition", "python")` and asserts `Some("decorators")`. This is the M36.2 acceptance criterion and it is honest: the function checks `decorators::NODE_KINDS.contains(&node_kind)`, and `"decorated_definition"` is now in that slice. Pre-existing drift-gate and doc-parity tests unchanged. **PASS.**

**`crates/sdivi-lang-typescript/tests/extract_behavior.rs`**
No M36.2 changes. Pre-existing M36.1 decorator tests exercise `TypeScriptAdapter.parse_file()` with real NestJS-shaped fixtures. `nestjs_shaped_controller_yields_multiple_decorator_hints` uses `>= 3` (not `== 3`) — defensively correct since the parser may emit additional structural hints. All assertions are grounded in fixture content. **PASS (no new M36.2 content; tester scope claim is misleading — see SCOPE finding).**

**`crates/sdivi-lang-javascript/tests/extract_behavior.rs`**
No M36.2 changes. Pre-existing vacuous-guard issue flagged under COVERAGE. Negative-path test (`file_with_no_decorators_produces_no_decorator_hints`) is concrete and correct. **PASS with carried-forward MEDIUM gap.**

---

### Rubric Scorecard

| # | Criterion | Verdict | Notes |
|---|---|---|---|
| 1 | Assertion Honesty | PASS | All new assertions derive from real function calls. No tautologies. |
| 2 | Edge Case Coverage | PASS | Happy path, stacked decorators, integration scenario, and negative case all covered for Python. JS vacuous-guard gap carried from M36.1. |
| 3 | Implementation Exercise | PASS | Real adapters called; no mocks at any level. |
| 4 | Test Weakening | PASS | All tester changes are additive. Rename in `decorators.rs` is a correct update. |
| 5 | Naming and Intent | PASS | All new test names encode scenario + expected outcome. |
| 6 | Scope Alignment | CONCERNS | Orphan claims are false positives (HIGH). Tester lists pre-existing TS/JS tests as new work (MEDIUM). |
| 7 | Test Isolation | PASS | All new tests use inline fixtures. No reads of mutable run artifacts. |
