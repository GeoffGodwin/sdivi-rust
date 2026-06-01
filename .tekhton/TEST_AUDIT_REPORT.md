## Test Audit Report

### Audit Summary
Tests audited: 1 file, 10 test functions
(`crates/sdivi-patterns/tests/dispatch_disjointness_supplement.rs`: 10 functions — 8 pre-existing, 2 new)
Verdict: PASS

### Findings

#### SCOPE: Shell-detected orphan claims are false positives
- File: crates/sdivi-patterns/tests/dispatch_disjointness_supplement.rs (all lines)
- Issue: The "Shell-Detected Orphans (pre-verified)" section claims this file (listed twice)
  imports the deleted module `.tekhton/.commit_decision`. Manual inspection confirms the only
  `use` declarations in the file are:
  ```
  use sdivi_patterns::queries::classify_hint;
  use sdivi_patterns::PatternHintInput;
  ```
  `.tekhton/.commit_decision` is a pipeline state file, not a Rust module — it cannot appear
  as a Rust `use` path. This is the same class of false positive identified in the M43 audit
  (see prior TEST_AUDIT_REPORT.md). No orphaned tests exist in the file. All imports resolve
  to current public API.
- Severity: LOW
- Action: Disregard the orphan warnings. No changes to the test file are needed. Investigate
  the orphan-detection script's pattern-matching heuristic — this false positive has recurred
  across consecutive milestone audits.

### No Issues Found In

The following rubric points were checked and found clean:

**Assertion Honesty — PASS**
All 10 assertions derive expected values from real implementation logic, verified against the
source regexes:

- `reduce_is_collection_pipelines`, `some_is_collection_pipelines`, `every_is_collection_pipelines`,
  `flat_is_collection_pipelines`, `find_index_is_collection_pipelines`: The
  `collection_pipelines::matches_callee` member-call regex covers `.reduce(`, `.some(`,
  `.every(`, `.flat(`, and `.findIndex(` — all five assertions are grounded in the
  implementation. These five entries were confirmed trimmed from `dispatch_disjointness.rs`
  at M43 and are correctly restored here.

- `fmt_print_is_logging_go`, `fmt_errorf_is_logging_go`: Go logging regex
  `^fmt\.(Print|Println|Printf|Errorf|Fprint|Sprint)` in `logging::matches_callee` matches
  both callee texts. CALL_DISPATCH P8 (logging) is reached before P9 (data_access). No
  hard-coded magic values.

- `promise_all_settled_is_concurrency_in_dispatch` (NEW, line 76): Calls `classify_hint`
  with node kind `"call_expression"` and text `"Promise.allSettled([p1, p2])"` in JavaScript.
  `concurrency::TS_JS_RE = r"^Promise\.(all|allSettled|race|any)\("` matches at the
  `allSettled` branch. No prior representative of `allSettled` existed in the main
  CORPUS — this test closes a genuine gap.

- `asyncio_create_task_is_concurrency_in_dispatch` (NEW, line 86): Calls `classify_hint`
  with node kind `"call"` (Python AST node kind) and text `"asyncio.create_task(coro())"`.
  `concurrency::PYTHON_RE = r"^asyncio\.(gather|create_task|wait|as_completed|run)\("` matches
  at the `create_task` branch. The use of `"call"` (not `"call_expression"`) is deliberate
  and correct — both arms share the same CALL_DISPATCH handling in `mod.rs:177`
  (`"call_expression" | "call" =>`). This exercises the `call` arm specifically and is a
  reasonable documentation of the Python node-kind variant.

- `unrecognised_callee_returns_empty` (line 95): `Math.sqrt(x)` in TypeScript is verified
  against all 11 active CALL_DISPATCH entries (P1–P11). None match, so `classify_hint`
  returns empty Vec. The assertion includes a descriptive failure message `{r:?}`. PASS.

**Edge Case Coverage — PASS**
Suite covers: five collection_pipelines methods absent from the main corpus; two Go fmt
logging variants absent from the main corpus; two new M44 concurrency callee variants
(allSettled and create_task) absent from the main corpus; one genuinely unrecognized callee
returning empty Vec. Positive and negative paths are both represented.

**Implementation Exercise — PASS**
All 10 tests call `classify_hint` directly through the real implementation. No mocks, no
stubs. The call chain exercises `CALL_DISPATCH` from `mod.rs`, which in turn calls
`concurrency::matches_callee`, `collection_pipelines::matches_callee`, and
`logging::matches_callee` respectively — all real regex evaluation paths.

**Test Weakening Detection — PASS**
The M43 audit established 8 pre-existing functions in this file. The current file has 10
functions — the 2 additions are `promise_all_settled_is_concurrency_in_dispatch` and
`asyncio_create_task_is_concurrency_in_dispatch`. No pre-existing test was removed. No
assertion was broadened (e.g., no `assert!(r == "collection_pipelines")` → `assert!(!r.is_empty())`
substitution). Assertions use `assert_eq!(r, vec!["concurrency"])` — exact match, not a
weakened contains/prefix check.

**Test Naming and Intent — PASS**
All 10 function names encode both the scenario and the expected outcome:
`reduce_is_collection_pipelines`, `fmt_errorf_is_logging_go`,
`promise_all_settled_is_concurrency_in_dispatch`, `asyncio_create_task_is_concurrency_in_dispatch`,
`unrecognised_callee_returns_empty`. No generic names (`test_1`, `test_thing`).

**Scope Alignment — PASS**
(False orphan claim notwithstanding — see finding above.)
All imports resolve to current public API. `sdivi_patterns::queries::classify_hint` and
`sdivi_patterns::PatternHintInput` exist and are unchanged by M44. The `concurrency` module
referenced indirectly through the dispatch chain is newly introduced by M44 and is present.

**Test Isolation — PASS**
The file defines its own `hint()` factory function (lines 14–19) that constructs
`PatternHintInput` inline. No reads of `.tekhton/` reports, pipeline logs, `package.json`,
`Cargo.lock`, or any mutable project state. Pass/fail outcome is fully determined by the
function under test and the inline literal inputs.

### Pre-Existing Issues (out of M44 scope — no action required)
- `wasm_package_json_version_matches_workspace`: `package.json` stranded at 0.2.23 vs
  workspace 0.2.35. Pre-existing; not introduced by M44. Noted in prior audits.
- `ALL_CATEGORIES` doc comment in `mod.rs` states only `logging` is callee-only via
  `classify_hint`, but several categories (including `concurrency` Go path) use both paths.
  Carry-over documentation drift; not introduced by M44.
- `list_categories_wasm_export_returns_eight_categories` function name in `m23_native.rs`
  is a historical artifact (body asserts 18); acknowledged by coder summary.
