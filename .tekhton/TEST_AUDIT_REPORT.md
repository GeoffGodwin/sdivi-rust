## Test Audit Report

### Audit Summary
Tests audited: 1 file, 4 test functions
(`crates/sdivi-patterns/tests/dispatch_disjointness.rs`)
Verdict: PASS

---

### Findings

#### COVERAGE: `corpus_resolves_identically_for_call_node_kind` delegates correctness to the first test without annotation
- File: `crates/sdivi-patterns/tests/dispatch_disjointness.rs:136`
- Issue: The test uses `_expected` and only asserts that the `call` and `call_expression` paths agree with each other — it does not independently verify the result is correct. This is intentional delegation to `corpus_resolves_to_expected_category`, but if the first test is ever removed or the corpus shrinks, this test continues passing while providing no correctness signal. The delegation intent is not documented in the test body.
- Severity: LOW
- Action: Add a one-line comment inside the test body: `// Correctness of the result is checked by corpus_resolves_to_expected_category; this test only verifies routing parity.`

#### COVERAGE: Java and Rust always-return-false paths in `data_access` are implicitly covered but unlabelled
- File: `crates/sdivi-patterns/tests/dispatch_disjointness.rs:103`
- Issue: `data_access::matches_callee` documents that Rust and Java always return `false` in v0. The CORPUS entry `("MyClass.method()", "java", "")` exercises this path but is labelled only as "Unrecognised — classify_hint must return empty Vec". If a future coder adds Java data-access regexes, they will not find an explicit fixture pinning the always-false contract and may not know coverage is expected.
- Severity: LOW
- Action: Add a brief comment on the relevant corpus entries (or a separate unrecognised-by-design block) noting that Java and Rust return `false` from `data_access::matches_callee` by design in v0. No additional test function needed.

#### COVERAGE: `all_matching_categories` helper hardcodes P1/P8/P9 rather than iterating `CALL_DISPATCH`
- File: `crates/sdivi-patterns/tests/dispatch_disjointness.rs:28`
- Issue: If a future milestone inserts a new category into `CALL_DISPATCH` without also updating `all_matching_categories`, the `no_undocumented_overlaps_in_corpus` test will silently miss overlaps between the new category and existing ones. The TODO comment on lines 25–27 acknowledges this and gives correct instructions for M35.
- Severity: LOW
- Action: The TODO comment is sufficient mitigation. When P2 lands (M35), the milestone author must follow the TODO instruction before merging. No change required in this audit cycle.

---

### Rubric Scorecard

| # | Criterion | Verdict | Notes |
|---|---|---|---|
| 1 | Assertion Honesty | PASS | All four tests call real functions with corpus-derived inputs; no hard-coded magic values; `vec![]` and `vec![winner]` assertions tie directly to CALL_DISPATCH implementation |
| 2 | Edge Case Coverage | PASS | Empty-result path covered by 4 CORPUS entries; both winner and loser verified for each KNOWN_OVERLAPS entry; `call`/`call_expression` parity verified across full corpus |
| 3 | Implementation Exercise | PASS | `classify_hint`, `async_patterns::matches_callee`, `logging::matches_callee`, `data_access::matches_callee` all called on real types with no mocking |
| 4 | Test Weakening | PASS | Tester added one KNOWN_OVERLAPS entry and one CORPUS row for P8>P9 overlap; no existing assertions were broadened or removed |
| 5 | Naming and Intent | PASS | All four names encode the scenario and expected outcome |
| 6 | Scope Alignment | PASS | All imports resolve to current symbols (`classify_hint`, `async_patterns`, `logging`, `data_access`, `PatternHintInput`); no deleted or renamed items referenced |
| 7 | Test Isolation | PASS | Only const tables and real function calls; no file I/O, no `.tekhton/` reads, no dependency on prior pipeline state or run artifacts |

---

### Tester Claim Verification

**Claim 1 — Added TODO comment to `all_matching_categories`.**
Confirmed at lines 25–27 of `dispatch_disjointness.rs`. ✓

**Claim 2 — Documented and tested P8>P9 (logging beats data\_access) overlap.**
Confirmed: second `KNOWN_OVERLAPS` entry at lines 62–67 (`logger.get("x")`, typescript, logging wins over data\_access). CORPUS entry at line 83. Mechanically exercised by both `no_undocumented_overlaps_in_corpus` and `known_overlaps_winner_matches_dispatch_order`.

The P8>P9 overlap claim is factually correct:
- `logging::matches_callee("logger.get(\"x\")", "typescript")` matches `^(console|logger|log)\.` → `true`.
- `data_access::matches_callee("logger.get(\"x\")", "typescript")` matches `\b(get)\(` (word boundary before `get` is satisfied by the preceding `.`) → `true`.
- `classify_hint` returns `vec!["logging"]` because `logging` (P8) precedes `data_access` (P9) in `CALL_DISPATCH`. ✓
