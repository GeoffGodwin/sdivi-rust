## Test Audit Report

### Audit Summary
Tests audited: 3 files, 33 test functions
Verdict: CONCERNS

Files:
- `crates/sdivi-patterns/tests/classify_hint.rs` — 26 tests (22 from coder + 4 tester-added disjoint-invariant tests)
- `crates/sdivi-patterns/tests/prop_classify_hint.rs` — 3 proptest cases
- `crates/sdivi-pipeline/tests/snapshot_m32_unchanged.rs` — 4 tests

---

### Findings

#### INTEGRITY: Vacuous test makes no assertions
- File: `crates/sdivi-pipeline/tests/snapshot_m32_unchanged.rs:73`
- Issue: `m32_different_seeds_may_differ` calls `pipeline_json(42)` and `pipeline_json(99)` then discards both results with `let _ = (seed_a, seed_b)`. The function body makes zero assertions — it passes as long as the pipeline does not panic. The test name encodes an expected outcome ("may differ") that is never checked. The inline comment explicitly acknowledges this: "We don't assert they MUST differ." The only implicit claim is "seed 99 does not panic," which will never catch a regression. This matches the rubric flag: a test that always passes regardless of implementation behavior.
- Severity: HIGH
- Action: Either (a) remove the test entirely — the infrastructure is already exercised by `m32_pipeline_output_byte_identical_for_same_params`, or (b) rename to a truthful name such as `m32_pipeline_does_not_panic_with_alternate_seed` and add a comment documenting it as a smoke test only. Do NOT add an assertion that seeds 42 and 99 produce different JSON — the `simple-rust` fixture has no graph edges, so Leiden output is seed-independent and such an assertion would be fragile.

#### SCOPE: Shell-detected orphan report is a false positive (all 3 files)
- File: `crates/sdivi-patterns/tests/prop_classify_hint.rs`, `crates/sdivi-patterns/tests/classify_hint.rs`, `crates/sdivi-pipeline/tests/snapshot_m32_unchanged.rs`
- Issue: The pre-verified orphan detection claims all three test files "import deleted module `.tekhton/.commit_decision`". Inspection of all three files in full shows no such dependency. Every `use` statement references valid crate symbols (`sdivi_patterns::queries::*`, `sdivi_pipeline::{Pipeline, WriteMode}`, etc.). `.tekhton/.commit_decision` is not a valid Rust module path and cannot appear as a Rust import. The detection script appears to be doing raw path-string matching against metadata files rather than analyzing actual Rust `mod`/`use` declarations.
- Severity: LOW
- Action: No changes needed to the test files. The orphan detection script should be rewritten to operate on actual Rust module declarations, not filesystem path strings.

#### NAMING: Vacuous test name misleads reviewers
- File: `crates/sdivi-pipeline/tests/snapshot_m32_unchanged.rs:73`
- Issue: `m32_different_seeds_may_differ` implies a comparison between two output values. The body makes no such comparison. This is the naming consequence of the INTEGRITY finding above.
- Severity: LOW
- Action: Address together with the INTEGRITY finding — rename to reflect the actual scope.

---

### Point-by-Point Rubric Results

#### 1. Assertion Honesty

**classify_hint.rs** — PASS. All assertions derive from real function outputs. Expected values (`vec!["logging"]`, `vec!["data_access"]`, `vec!["async_patterns"]`, `Vec::<&str>::new()`) are grounded in the regex patterns in the implementation. Anchor behavior is explicitly probed: `myconsole.log("x")` tests the `^(console|logger|log)\.` anchor; `myfmt.Println("x")` tests the `^fmt\.` anchor; `printer(x)` tests the `\b` guard in Python. Every positive case has a corresponding negative case.

**prop_classify_hint.rs** — PASS. Property tests compare `classify_hint` output against `category_for_node_kind` output computed at runtime — no hard-coded magic values. The `prop_text_does_not_affect_fall_through` property derives its equality claim from two runtime calls with different inputs, not a pre-baked expected value.

**snapshot_m32_unchanged.rs** — PARTIAL. Three of four tests are honest:
- `m32_pipeline_output_byte_identical_for_same_params`: asserts string equality of two real pipeline runs. Honest.
- `m32_pipeline_snapshot_has_no_logging_entry_in_catalog`: asserts absence of a `logging` key in real JSON output. Honest.
- `m32_pipeline_snapshot_has_expected_schema_version`: asserts `snapshot_version == "1.0"` against real serialized output. Honest.
- `m32_different_seeds_may_differ`: makes no assertions. See INTEGRITY finding above.

#### 2. Edge Case Coverage — PASS

**classify_hint.rs** covers: unrecognized callees (empty vec), unknown node kinds (empty vec), non-TS/JS languages for async_patterns (false), Rust data_access returns false by design, disjoint-regex invariant across TypeScript, Python, Go, and Rust (call and macro paths), symmetric-agreement invariant between `resource_management::excludes_callee` and `logging::matches_callee`.

**prop_classify_hint.rs** covers: unknown node kinds with arbitrary text (`prop_unknown_kind_falls_through_to_empty`), text-agnosticity for non-special kinds across all 6 languages (`prop_text_does_not_affect_fall_through`), all 11 known non-special kinds × 6 languages × 500 proptest cases.

**snapshot_m32_unchanged.rs** covers: determinism (same-seed byte equality), schema version guard, logging-bucket absence. Only one fixture (`simple-rust`) is exercised; this is appropriate for a regression guard focused on proving the pipeline path was not changed.

#### 3. Implementation Exercise — PASS

All tests call real functions with no mocking: `classify_hint`, `data_access::matches_callee`, `logging::matches_callee`, `async_patterns::matches_callee`, `resource_management::excludes_callee`, `category_for_node_kind`, `Pipeline::snapshot_with_mode`. `WriteMode::EphemeralForCheck` and `snapshot_with_mode` are confirmed pre-existing public APIs (exported from `sdivi_pipeline::lib.rs:33`; used by multiple pre-existing test files).

#### 4. Test Weakening Detection — N/A

All three test files are newly created (`??` in git status). There are no prior versions to weaken. The tester's additions to `classify_hint.rs` (the four `disjoint_regex_invariant_for_*` tests) extend coverage beyond the coder's 22 tests without removing or broadening any existing assertion.

#### 5. Test Naming and Intent — PASS (with one exception)

All test names follow the `<scenario>_<expected_outcome>` or `<function>_<behavior>_<language>` pattern:
- `classify_hint_async_beats_logging` — encodes priority rule
- `disjoint_regex_invariant_for_rust_macro_invocations` — encodes invariant and scope
- `m32_pipeline_snapshot_has_no_logging_entry_in_catalog` — encodes condition and expected absence

Exception: `m32_different_seeds_may_differ` — see NAMING finding above.

#### 6. Scope Alignment — PASS

All imports reference valid symbols confirmed present in the codebase. `sdivi_patterns::queries::{async_patterns, classify_hint, data_access, logging, resource_management}` and `sdivi_patterns::PatternHintInput` are confirmed exported in the coder-modified files. `sdivi_patterns::queries::category_for_node_kind` is confirmed present in `queries/mod.rs:80`. No stale references to renamed or removed symbols.

The three shell-detected orphan claims have no basis in the actual source files (see SCOPE finding).

#### 7. Test Isolation — PASS

- `classify_hint.rs` and `prop_classify_hint.rs`: pure in-process function calls, no I/O.
- `snapshot_m32_unchanged.rs`: reads `tests/fixtures/simple-rust` via compile-time `CARGO_MANIFEST_DIR` — a committed, read-only fixture tree, not a mutable project state file. No reads from `.tekhton/`, `.sdivi/`, `.claude/`, or any pipeline output file. `WriteMode::EphemeralForCheck` prevents the test from writing to `.sdivi/snapshots/`, so the test leaves no side effects. Isolation is sound.
