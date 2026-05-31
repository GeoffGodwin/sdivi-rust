## Test Audit Report

### Audit Summary
Tests audited: 3 files, 31 test functions
Verdict: PASS

Files:
- `crates/sdivi-core/tests/category_contract_m38.rs` — 13 test functions (new file)
- `crates/sdivi-patterns/tests/dispatch_disjointness.rs` — 4 test functions (modified)
- `bindings/sdivi-wasm/tests/wasm_smoke.rs` — 14 test functions (modified)

### Findings

#### SCOPE: Shell orphan detector produced false positives for all three files
- File: `crates/sdivi-core/tests/category_contract_m38.rs`
- File: `crates/sdivi-patterns/tests/dispatch_disjointness.rs`
- File: `bindings/sdivi-wasm/tests/wasm_smoke.rs`
- Issue: The pre-verified shell orphan report claims all three files import the deleted
  module `.tekhton/.commit_decision`. After reading each file in full, none of them
  contain any reference to that path. A workspace-wide grep confirms the string
  `commit_decision` appears only in `.tekhton/TEST_AUDIT_REPORT.md` itself — not in
  any test source file. The orphan detector processed incorrect or stale input. These
  files are valid and must NOT be modified on the basis of this report.
- Severity: LOW
- Action: Investigate the orphan detection tool's input snapshot. Do not remove or
  alter any of these test files on the basis of the orphan claim. No code change needed.

#### COVERAGE: Scoped negative assertion allows undetected accidental matches
- File: `crates/sdivi-core/tests/category_contract_m38.rs:124`
- Issue: `bare_string_method_does_not_match_schema_validation` asserts
  `!result.contains(&"schema_validation")` rather than `result.is_empty()`. The
  assertion verifies only that `schema_validation` is absent, not that `.string()` is
  completely unclassified. In practice the result is empty — verified against
  `data_access.rs` (TS/JS/Go regex does not match `.string()`) and `logging.rs` —
  but the weaker form would silently pass if `.string()` accidentally matched another
  category in a future regression.
- Severity: LOW
- Action: Tighten to `assert!(result.is_empty(), ".string() on arbitrary receiver must
  return [] not {result:?}")` to catch cross-category accidental matches.

### Per-File Assessment

#### `crates/sdivi-core/tests/category_contract_m38.rs`

**Assertion Honesty — PASS.** All 13 assertions are grounded in real regex behavior.
`z.object({})` verified against `TS_JS_RE = ^(z|yup|v|s)\.\w|\.safeParse\(`;
`Math.max(a,b)` correctly does not match (no `^z/yup/v/s\.` prefix, no `.safeParse(`);
Pydantic calls verified against `PYTHON_RE = \bField\(|\bconstr\(|\bconint\(`. No
hard-coded magic values or tautologies.

**Edge Case Coverage — PASS.** Covers: happy-path positives for all four TS/JS libraries
(Zod, Yup, Valibot, Superstruct) and all three Pydantic functions (`Field`, `constr`,
`conint`); `.safeParse(` positive; bare-receiver negative; wrong-language negatives for
`rust`, `go`, `java`; disjointness assertion against `data_access` for `.safeParse(`.

**Implementation Exercise — PASS.** Tests call the real `classify_hint` and
`sdivi_core::list_categories` with no mocking of any kind.

**Test Weakening — N/A.** New file; no prior tests to weaken.

**Naming — PASS.** All names encode scenario and expected outcome
(e.g., `pydantic_conint_is_schema_validation`,
`other_languages_do_not_match_schema_validation`,
`schema_validation_and_data_access_are_disjoint_for_parse`).

**Scope Alignment — PASS.** All imports (`sdivi_patterns::queries::classify_hint`,
`sdivi_patterns::PatternHintInput`, `sdivi_core::list_categories`) exist in the current
codebase. No stale references.

**Isolation — PASS.** All test data constructed inline via the `call_hint` helper or
direct `PatternHintInput` struct literals. No mutable project files read.

---

#### `crates/sdivi-patterns/tests/dispatch_disjointness.rs`

**Assertion Honesty — PASS.** Corpus expected values are derived from real regex
behavior. The `schema_validation` entries (`z.object`, `yup.object().shape`, `v.object`,
`s.object`, `UserSchema.safeParse`, `Field`, `constr`, `conint`) verified against the
actual regex tables in `schema_validation.rs`. The negative `SomeClass.parse(x)` correctly
returns empty: `SomeClass` begins with uppercase `S`, so `^(z|yup|v|s)\.\w` does not
match; `.parse(` is not `.safeParse(`.

**Edge Case Coverage — PASS.** `no_undocumented_overlaps_in_corpus` exercises the overlap
detection path. `corpus_resolves_identically_for_call_node_kind` tests the
`call`/`call_expression` equivalence invariant (both are handled in the same match arm in
`mod.rs:157`). `known_overlaps_winner_matches_dispatch_order` verifies both winner and
loser sides of each documented overlap. Multiple negative corpus entries (`""`) exercise
the empty-return path.

**Implementation Exercise — PASS.** Tests call `classify_hint` and individual
`matches_callee` functions directly against real regex tables.

**Test Weakening — PASS.** The tester-added `conint(gt=0)` corpus entry (line 119) and
`schema_validation` arm in `all_matching_categories` (line 33) strictly strengthen the
suite. No existing assertions were removed or broadened.

**Naming — PASS.** Function names describe the invariant tested
(`no_undocumented_overlaps_in_corpus`, `known_overlaps_winner_matches_dispatch_order`,
`corpus_resolves_identically_for_call_node_kind`).

**Scope Alignment — PASS.** `schema_validation` import added at line 14 resolves to the
new module. Dispatch comment "P1/P4/P6/P8/P9 active at M38" matches `CALL_DISPATCH` order
in `mod.rs:109-115`.

**Isolation — PASS.** `CORPUS` and `KNOWN_OVERLAPS` are static compile-time constants.
No mutable project files read.

---

#### `bindings/sdivi-wasm/tests/wasm_smoke.rs`

**Assertion Honesty — PASS.** `list_categories_returns_schema_version_and_expected_count`
asserts `catalog.categories.len() == 12` and individual `names.contains(...)` for all 12
categories. The count of 12 is derived from `CATALOG_ENTRIES.len()` in `categories.rs`
(confirmed 12 entries). The tester-added assertions for `resource_management`,
`state_management`, and `type_assertions` fill a pre-existing coverage gap where those
categories were in `CATALOG_ENTRIES` but not individually name-checked.

**Edge Case Coverage — N/A for the modified function.** The function tests the catalog
contract; completeness of the name list is the meaningful invariant, and all 12 names
are now individually asserted.

**Implementation Exercise — PASS.** Calls the real WASM-bound `list_categories()`;
no mocking.

**Test Weakening — PASS.** Modification added 3 `assert!(names.contains(...))` calls.
No assertions were removed or relaxed.

**Naming — PASS.** `list_categories_returns_schema_version_and_expected_count` describes
both sub-invariants (schema version and category count) under test.

**Scope Alignment — PASS.** All WASM imports resolve. `null_safety` and
`schema_validation` added to name assertions match the updated `CATALOG_ENTRIES`.

**Isolation — PASS.** No mutable project files read. All test fixtures constructed inline.

---

### Pre-Existing Issues (out of M38 scope — no action required)
- `wasm_package_json_version_matches_workspace`: `package.json` stranded at 0.2.23,
  workspace at 0.2.29. Pre-existing; not introduced by M38.
- `null_safety_node_kinds_do_not_match_non_ts_js_languages` in `tests.rs`: body asserts
  a match while the name implies the opposite. Pre-existing from M37; out of audit scope.
