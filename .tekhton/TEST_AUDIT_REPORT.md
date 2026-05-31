## Test Audit Report

### Audit Summary
Tests audited: 1 file, 6 test functions (`crates/sdivi-lang-typescript/tests/null_safety_hints.rs`)
Verdict: PASS

---

### Findings

#### SCOPE: Orphan detection false positive — no action needed
- File: `crates/sdivi-lang-typescript/tests/null_safety_hints.rs`
- Issue: The pre-verified orphan list states this file "imports deleted module `.tekhton/.commit_decision`". Reading the actual file at lines 1–119 reveals no such import. The only `use` declarations are `sdivi_lang_typescript::TypeScriptAdapter`, `sdivi_parsing::adapter::LanguageAdapter`, and `std::path::Path` — all live, current crates. This is a repeat false positive from the same orphan detection script that produced incorrect results across all five M36.2 files. The underlying test file is correct; no action is needed here.
- Severity: LOW (defect is in the detection tooling, not the test code)
- Action: No change to the test file. Orphan detection script should be fixed separately — this is its third consecutive run producing a false positive.

#### NAMING: Acceptance criterion name overstates assertion precision
- File: `crates/sdivi-lang-typescript/tests/null_safety_hints.rs:48`
- Issue: `ts_fixture_with_optional_chain_and_non_null_yields_two_null_safety_instances` implies exactly two total hints. The assertions at lines 62–69 are `opt_count >= 1` and `nne_count >= 1` — correct and robust, but the combined lower bound is `>= 2 total`, not `== 2`. The name sets an exactness expectation the assertions do not enforce.
- Severity: LOW
- Action: Either rename to `ts_fixture_with_optional_chain_and_non_null_yields_both_hint_kinds`, or add `assert_eq!(opt_count + nne_count, 2)` if the intent is to pin the precise output of this two-line fixture.

#### None — all other rubric categories are clean

---

### Per-Rubric Assessment

**1. Assertion Honesty — PASS**
Every assertion calls `parse_ts()` which invokes `TypeScriptAdapter.parse_file()` → `extract::collect_hints()` → real tree-sitter parse. No hard-coded magic values, no tautologies (`assert!(true)`, `assert_eq!(x, x)`). The node-kind strings asserted (`"optional_chain"`, `"non_null_expression"`) appear in the implementation at `extract.rs:19-20` (`PATTERN_KINDS`) and `null_safety.rs:49` (`NODE_KINDS`). The `>= 2` assertion in `optional_chain_member_access_variants_captured` (line 82) and `chained_optional_chain_produces_multiple_nodes` (line 100) reflects documented grammar behavior — each `optional_chain` node emitted by the tree-sitter parser is a distinct hint, which the module doc at `null_safety.rs:15-20` explicitly states.

**2. Edge Case Coverage — PASS**
- Happy path single-kind: `optional_chain_captured_as_pattern_hint` (line 12), `non_null_expression_captured_as_pattern_hint` (line 30)
- Multi-kind acceptance criterion: `ts_fixture_with_optional_chain_and_non_null_yields_two_null_safety_instances` (line 48)
- Grammar variant documentation: `optional_chain_member_access_variants_captured` (line 73) — documents that `obj?.field` and `arr?.[0]` both emit `optional_chain` and that `fn?.()` does NOT (aligns with coder's design observation in CODER_SUMMARY.md)
- Counting semantics pin: `chained_optional_chain_produces_multiple_nodes` (line 89) — added by tester, pins per-node counting behavior documented in MIGRATION_NOTES.md
- Negative/zero: `file_with_no_optional_chain_produces_no_null_safety_hints` (line 107) — checks both node kinds are absent for plain member access

Ratio of non-happy-path tests: 2 of 6 (33%) — adequate for a node-kind feature test with documented grammar caveats.

**3. Implementation Exercise — PASS**
All six tests call `TypeScriptAdapter.parse_file()`, a real adapter backed by a live tree-sitter grammar. `collect_hints()` in `extract.rs:113-136` is the real production code path. No mocking at any level.

**4. Test Weakening Detection — PASS**
The tester added exactly one test (`chained_optional_chain_produces_multiple_nodes`, lines 89–104). The remaining five tests are the coder's delivery, unchanged. No assertions were removed, broadened, or reclassified. The new test adds a positive constraint (`>= 2`) that would fail if future grammar versions changed node emission semantics — strictly additive coverage.

**5. Naming and Intent — PASS**
Five of six names clearly encode both scenario and expected outcome:
- `optional_chain_captured_as_pattern_hint` ✓
- `non_null_expression_captured_as_pattern_hint` ✓
- `optional_chain_member_access_variants_captured` ✓
- `chained_optional_chain_produces_multiple_nodes` ✓
- `file_with_no_optional_chain_produces_no_null_safety_hints` ✓
One name is imprecise: `ts_fixture_with_optional_chain_and_non_null_yields_two_null_safety_instances` (see NAMING finding above — LOW).

**6. Scope Alignment — PASS (with false-positive note)**
All symbols in the test file exist in the current codebase. Both node kinds exercised by the tests (`optional_chain`, `non_null_expression`) are present in `PATTERN_KINDS` at `extract.rs:19-20` and in `null_safety::NODE_KINDS` at `null_safety.rs:49`. No references to deleted, renamed, or refactored items. The orphan claim is a detection script false positive as noted above.

**7. Test Isolation — PASS**
All six tests create fixtures as inline string literals within the test function body. No test reads from `.tekhton/` reports, snapshot directories, Cargo build artifacts, config files, or any other mutable project state. `parse_ts()` at line 7–9 is a local helper that takes a `&str` and returns a `FeatureRecord` — no shared state, no filesystem interaction.
