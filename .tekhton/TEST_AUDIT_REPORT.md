## Test Audit Report

### Audit Summary
Tests audited: 1 file (`bindings/sdi-wasm/tests/wasm_smoke.rs`), 16 test functions
Implementation files cross-referenced: `bindings/sdi-wasm/src/exports.rs`,
`bindings/sdi-wasm/src/types.rs`, `crates/sdi-core/src/lib.rs`,
`crates/sdi-core/src/compute/boundaries.rs`, `crates/sdi-core/src/compute/coupling.rs`,
`crates/sdi-core/src/compute/thresholds.rs`, `crates/sdi-snapshot/src/delta.rs`,
`crates/sdi-snapshot/src/trend.rs`, `crates/sdi-snapshot/src/boundary_inference.rs`,
`crates/sdi-patterns/src/fingerprint.rs`.

Verdict: CONCERNS

---

### Findings

#### INTEGRITY: test_compute_boundary_violations_with_spec asserts against a stub that always returns 0
- File: `bindings/sdi-wasm/tests/wasm_smoke.rs:61` (`test_compute_boundary_violations_with_spec`)
- Issue: The test constructs a two-boundary spec (`core` / `data`) and a graph with an edge
  lib.rs → models.rs, reasons about whether the edge is a violation, and asserts
  `result.violation_count == 0`. However, `compute_boundary_violations` in
  `crates/sdi-core/src/compute/boundaries.rs:224–236` is an explicit stub: the `spec`
  parameter is named `_spec` and the function body always returns
  `Ok(BoundaryViolationResult { violation_count: 0, violations: vec![] })` regardless
  of input. The assertion always passes, and the test cannot detect any future regression
  when the real boundary violation logic (deferred to Milestone 10) is wired up. This is
  a structurally always-passing assertion that misleads reviewers into believing boundary
  logic is exercised.
- Severity: HIGH
- Action: Mark the test `#[ignore]` with a comment referencing the stub and the milestone
  that will complete the implementation, OR replace the assertion with a `TODO` comment
  that explicitly acknowledges the stub, e.g.:
  ```rust
  // TODO(M10): compute_boundary_violations is a stub; rewrite this test with
  // a concrete violation scenario once the real implementation ships.
  assert_eq!(result.violation_count, 0); // trivially true against current stub
  ```
  Do not implement the missing logic to make the test non-trivial — tests follow code.

---

#### INTEGRITY: test_compute_boundary_violations_empty_spec also exercises the stub
- File: `bindings/sdi-wasm/tests/wasm_smoke.rs:53` (`test_compute_boundary_violations_empty_spec`)
- Issue: Same stub root cause as above. An empty spec must always yield zero violations
  in any complete implementation, so the assertion is semantically correct, but the test
  cannot distinguish "stub returned 0" from "real logic returned 0 for an empty spec."
  This is lower severity than the non-trivial spec case because the expected value (0)
  is correct regardless of whether the spec is consulted.
- Severity: LOW
- Action: No change required to the assertion. Add a comment noting the stub limitation,
  consistent with the HIGH finding above.

---

#### COVERAGE: normalize_and_hash tests do not pin the expected hash value
- File: `bindings/sdi-wasm/tests/wasm_smoke.rs:172` (`test_normalize_and_hash_deterministic`)
         `bindings/sdi-wasm/tests/wasm_smoke.rs:191` (`normalize_hash_deterministic`)
- Issue: Both tests verify only that the returned string is 64 characters and all-hex.
  Neither asserts the concrete expected digest for the `"try_expression"` + empty-children
  input. Rule 23 and KDD-19 state that `normalize_and_hash` must produce a byte-identical
  digest across all platforms (WASM and native). Without a pinned expected value,
  a regression from changing `FINGERPRINT_KEY` in `sdi-patterns::fingerprint` (Rule 19)
  or from modifying the normalize algorithm in `crates/sdi-core/src/compute/normalize.rs`
  would not be detected by these tests. The CI cross-platform grep of `CI_HASH:` output
  checks same-run consistency, but not stability across code changes.
- Severity: MEDIUM
- Action: Compute the expected hash once (run `normalize_and_hash("try_expression", vec![])`
  locally), record it as a constant in the test, and add:
  ```rust
  const EXPECTED_TRY_EXPRESSION_HASH: &str = "<64-char hex>";
  assert_eq!(h1, EXPECTED_TRY_EXPRESSION_HASH);
  ```
  This makes the test a regression guard for the fingerprint key and algorithm.

---

#### NAMING: normalize_hash_deterministic lacks the test_ prefix used by all other test functions
- File: `bindings/sdi-wasm/tests/wasm_smoke.rs:191` (`normalize_hash_deterministic`)
- Issue: All 15 other test functions in the file use a `test_` prefix
  (e.g., `test_normalize_and_hash_deterministic`). The function at line 191 is named
  `normalize_hash_deterministic`. The `#[wasm_bindgen_test]` macro still executes it,
  so it is not dead, but the inconsistency reduces readability and may confuse future
  contributors about whether the function is a test or a helper.
- Severity: LOW
- Action: Rename to `test_normalize_hash_deterministic_ci_output`.

---

#### COVERAGE: no error-path tests for any exported function
- File: `bindings/sdi-wasm/tests/wasm_smoke.rs` (whole file)
- Issue: Every test exercises a happy path. None of the 16 tests verify that exported
  functions return a `JsError` on invalid input. Examples of untested error paths:
  - `compute_coupling_topology` / `detect_boundaries` with a node whose ID violates
    `validate_node_id` (e.g., an absolute path) — should return `AnalysisError::InvalidNodeId`.
  - `build_pattern_catalog` (called inside `assemble_snapshot`) with a 65-char or
    non-hex fingerprint string — should return the `JsError` from `PatternFingerprint::from_hex`.
  - `compute_delta` with a `JsValue` that is not a valid `Snapshot` JSON — should return
    a deserialization error.
- Severity: LOW
- Action: Add at minimum one `#[wasm_bindgen_test]` that calls an exported function with
  an invalid input and asserts the `Result` is `Err`. This validates that the WASM binding
  error-conversion path (`JsError::new`) works end-to-end and does not panic.

---

### Passing Items

**Assertion Honesty — PASS (except where noted).** The three primary assertions introduced
by the tester — `compute_delta` self-delta coupling == `Some(0.0)`, coupling delta ≈ 0.2
for densities 0.1→0.3, and coupling slope ≈ 0.2 for three linearly increasing snapshots —
all derive their expected values from the test's own inputs and the documented implementation
(`delta.rs:129`: `coupling_delta = Some(curr.graph.density - prev.graph.density)`;
`trend.rs:106–107`: `mean_slope` over density values). No magic constants.

**Test Weakening — PASS.** The audit context contains no prior version of this test file
(it is NEW per the coder summary). No existing assertions were modified or removed.

**Implementation Exercise — PASS.** Every test calls a real `#[wasm_bindgen]` export.
The conversion path (wrapper type → `to_core` → `sdi_core::*` → `from_core` → wrapper type)
is exercised end-to-end for `compute_coupling_topology`, `detect_boundaries`,
`compute_pattern_metrics`, `compute_thresholds_check`, `infer_boundaries`,
`normalize_and_hash`, `assemble_snapshot`, `compute_delta`, and `compute_trend`. No internal
dependencies are mocked.

**Test Naming — PASS (with one LOW exception above).** 15 of 16 functions follow the
`test_<scenario>_<expected_outcome>` convention and name the scenario and expectation clearly.

**Scope Alignment — PASS.** All imports resolve against current code. The deleted
`.tekhton/test_dedup.fingerprint` is not referenced. `WasmAssembleSnapshotInput.leiden_seed`
(added this run) is exercised in `make_assemble_input` at line 233 (`leiden_seed: Some(42)`),
consistent with the coder's change to `build_leiden_partition` using `leiden_seed` instead
of a hardcoded value.

**Test Isolation — PASS.** All tests construct their own in-memory inputs via factory
functions (`two_node_graph`, `default_leiden_cfg`, `make_assemble_input`). No test reads
`.tekhton/` files, build artifacts, or any mutable project state. `wasm_smoke.rs` has
no filesystem I/O.
