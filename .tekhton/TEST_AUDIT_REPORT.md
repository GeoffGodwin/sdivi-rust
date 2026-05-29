## Test Audit Report

### Audit Summary
Tests audited: 4 files, 42 test functions (including 3 property-test bodies × 500 cases each)
Verdict: CONCERNS

Files reviewed:
- `crates/sdivi-patterns/tests/prop_classify_hint.rs` — 3 property tests
- `crates/sdivi-patterns/tests/classify_hint.rs` — 21 unit tests
- `crates/sdivi-pipeline/tests/snapshot_m32_unchanged.rs` — 4 integration tests
- `crates/sdivi-patterns/tests/simple_go_fixture.rs` — 9 integration tests

---

### Findings

#### INTEGRITY: Pre-verified orphan claims are false positives — do not act on them
- File: All four audited test files (as named in the audit context orphan list)
- Issue: The audit context asserts 8 "Shell-Detected Orphans" (listed twice each)
  claiming every test file "imports deleted module `.tekhton/.commit_decision`".
  A direct grep of all four files for "commit_decision" and "stage_tester" returns
  zero matches. The files are valid Rust source; none contain a `use` or `include!`
  referencing that path. `.tekhton/.commit_decision` is a plain-text Tekhton state
  file — it has no valid Rust module path and cannot be imported by a Rust test.
  The detection script appears to be doing raw filesystem-path string matching
  against something other than Rust `use`/`mod` declarations.
  Acting on these claims would delete all four test files, eliminating coverage for
  the entire M33 behavioral change.
- Severity: HIGH
- Action: Discard the pre-verified orphan data. Do not remove any of the four test
  files. Fix the orphan-detection script to operate on actual Rust module declarations
  (`use` and `mod` statements), not filesystem path strings.

#### COVERAGE: Vacuous test makes zero assertions
- File: `crates/sdivi-pipeline/tests/snapshot_m32_unchanged.rs:73-80`
- Issue: `m32_different_seeds_may_differ` calls `pipeline_json(42)` and
  `pipeline_json(99)`, then discards both with `let _ = (seed_a, seed_b)`. The body
  makes zero assertions — it passes regardless of pipeline output as long as no panic
  occurs. The inline comment explicitly acknowledges this limitation. The test name
  implies a comparison that never happens. The only value is "does not panic with
  seed 99", which is already covered by `m32_pipeline_output_byte_identical_for_same_params`
  (which runs the pipeline twice with seed 42 and would surface a panic there too).
- Severity: MEDIUM
- Action: Either (a) remove the test — the pipeline-infrastructure smoke test is
  redundant given the existing determinism test — or (b) rename to a truthful name
  such as `pipeline_does_not_panic_with_alternate_seed` with an explicit comment
  documenting it as a no-assertion smoke test. Do NOT add an assertion that seeds 42
  and 99 produce different JSON; the `simple-rust` fixture has no graph edges so
  Leiden output is seed-independent for this fixture and such an assertion would be
  fragile.

#### NAMING: File and test names retain M32 prefix after M33 rebasing
- File: `crates/sdivi-pipeline/tests/snapshot_m32_unchanged.rs` (filename and lines 57, 114)
- Issue: The file retains the name `snapshot_m32_unchanged.rs` and contains tests
  named `m32_pipeline_output_byte_identical_for_same_params` and
  `m32_pipeline_snapshot_has_expected_schema_version`, but the file now primarily
  guards M33 behavior (the `m33_pipeline_snapshot_has_logging_entry_for_tracing_macros`
  test at line 91 is the most important assertion). The mix of M32 and M33 labels in a
  single file will confuse future readers about which milestone the suite protects.
  The behavioral content of the determinism and schema-version tests is correct.
- Severity: LOW
- Action: Rename the file to `snapshot_pipeline_regression.rs` (or similar) and drop
  the milestone prefix from the two remaining M32-named tests. The behavioral content
  of those tests does not need to change.

---

### Point-by-Point Rubric Results

#### 1. Assertion Honesty — PASS (with one exception)

**classify_hint.rs**: All assertions derive from real function outputs against values
grounded in the implementation's regex patterns. Anchor behavior is explicitly probed
(e.g. `myconsole.log("x")` verifies the `^` anchor on the TS/JS logging regex;
`myfmt.Println("x")` verifies Go's `^fmt\.` anchor; `printer(x)` verifies Python's
`\b` guard). Every positive case has a corresponding negative.

**prop_classify_hint.rs**: Property tests compare `classify_hint` output against
`category_for_node_kind` at runtime — no hard-coded magic values. The
`prop_text_does_not_affect_fall_through` property derives equality from two runtime calls
with different inputs, not a pre-baked constant.

**snapshot_m32_unchanged.rs**: Three of four tests are honest:
- `m32_pipeline_output_byte_identical_for_same_params` — real determinism assertion.
- `m33_pipeline_snapshot_has_logging_entry_for_tracing_macros` — real JSON key check.
- `m32_pipeline_snapshot_has_expected_schema_version` — real version string assertion.
- `m32_different_seeds_may_differ` — makes no assertion. See COVERAGE finding.

**simple_go_fixture.rs**: All assertions are real: category presence/absence in
`catalog.entries` and exact instance counts derived from the number of synthetic hints
provided. The mixed-fixture test (line 215) verifies the exact split (2 logging + 2
data_access + 2 dropped = 4 classified total).

#### 2. Edge Case Coverage — PASS

**classify_hint.rs**: Covers unrecognized callees (empty Vec), unknown node kinds (empty
Vec), async_patterns priority over logging/data_access, Rust data_access returning false
by design (v0 deferral documented in the implementation), disjoint-regex invariant across
TypeScript, Python, Go, and Rust (both call and macro_invocation paths), and the
symmetric-agreement invariant between `resource_management::excludes_callee` and
`logging::matches_callee` for 13 Rust macro samples including non-excluded cases.

**prop_classify_hint.rs**: Covers arbitrary unknown node kinds with arbitrary text,
text-agnosticity for non-special kinds across all 6 supported languages, and all 11
known non-special kinds × 6 languages.

**snapshot_m32_unchanged.rs**: Covers determinism across repeated runs, schema version
guard, and positive M33 logging presence. Only `simple-rust` is exercised, which is
appropriate for a determinism regression guard.

**simple_go_fixture.rs**: Covers per-callee routing for `fmt.Println`, `fmt.Printf`,
`fmt.Errorf`, `db.query`, `sql.Open`; non-matching drops for `os.Exit` and
`strings.Join`; a mixed-fixture test that exercises all three paths simultaneously; and
the `min_pattern_nodes` filter with a threshold that suppresses a single-instance entry.

#### 3. Implementation Exercise — PASS

All tests call real implementation code with no mocking of functions under test:
`classify_hint`, `category_for_node_kind`, `data_access::matches_callee`,
`logging::matches_callee`, `async_patterns::matches_callee`,
`resource_management::excludes_callee`, `build_catalog`, and
`Pipeline::snapshot_with_mode`. No test mocks the pipeline, catalog, or regex
functions.

#### 4. Test Weakening Detection — PASS

The replacement of `m32_pipeline_snapshot_has_no_logging_entry_in_catalog`
(asserted logging absent) with `m33_pipeline_snapshot_has_logging_entry_for_tracing_macros`
(asserts logging present) is a correct rebasing, not a weakening. The old assertion
described behavior that M33 intentionally changes. The new assertion encodes the new
expected behavior. The tester report documents this with `re-baselined in M33`. The
remaining two determinism and schema-version tests are unchanged and retain their
assertion strength.

`simple_go_fixture.rs` and `prop_classify_hint.rs` are new files with no prior
version to weaken.

#### 5. Test Naming and Intent — PASS (with one exception)

Test names across all four files encode scenario and expected outcome clearly:
`classify_hint_async_beats_logging`, `go_fmt_errorf_routes_to_logging_not_data_access`,
`prop_text_does_not_affect_fall_through`, `go_min_pattern_nodes_filter_drops_single_instance_logging`.
Exception: `m32_different_seeds_may_differ` — see NAMING finding.

#### 6. Scope Alignment — PASS

All imports reference symbols confirmed present in the implementation:
`sdivi_patterns::queries::{classify_hint, category_for_node_kind, async_patterns,
data_access, logging, resource_management}`, `sdivi_patterns::PatternHintInput`,
`sdivi_patterns::build_catalog`, `sdivi_config::PatternsConfig`,
`sdivi_parsing::feature_record::{FeatureRecord, PatternHint}`,
`sdivi_pipeline::{Pipeline, WriteMode}`. No stale references to renamed or removed
symbols were found.

The shell-detected orphan claims have no basis in the actual source files (see INTEGRITY
finding). These tests are not orphaned.

#### 7. Test Isolation — PASS

`classify_hint.rs` and `prop_classify_hint.rs`: pure in-process function calls, no I/O.

`simple_go_fixture.rs`: constructs `FeatureRecord` and `PatternHint` structs entirely
in memory, calls `build_catalog` directly, no filesystem access.

`snapshot_m32_unchanged.rs`: reads `tests/fixtures/simple-rust` via compile-time
`CARGO_MANIFEST_DIR` — a committed, stable fixture tree, not a mutable project state
file such as `.tekhton/CODER_SUMMARY.md` or pipeline run artifacts. Uses
`WriteMode::EphemeralForCheck` to prevent writing to `.sdivi/snapshots/`, leaving no
side effects. No reads from `.tekhton/`, `.sdivi/`, `.claude/logs/`, or any pipeline
output file. Isolation is sound.
