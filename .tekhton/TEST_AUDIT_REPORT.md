## Test Audit Report

### Audit Summary
Tests audited: 2 files, 21 test functions
Verdict: CONCERNS

Files audited:
- `crates/sdivi-core/tests/prop_thresholds.rs` (9 proptest functions; 4 added by tester)
- `bindings/sdivi-wasm/src/weight_keys.rs` `#[cfg(test)] mod tests` (12 unit tests; 5 added by tester)

---

### Findings

#### EXERCISE: Failing test exposes unimplemented infinity fix
- File: `bindings/sdivi-wasm/src/weight_keys.rs:172`
- Issue: `rejects_positive_infinity_weight` calls `unwrap_err()` expecting `parse_wasm_edge_weights`
  to reject `f64::INFINITY`. The implementation checks `weight.is_nan()` (line 25) and
  `weight < 0.0` (line 29) but has no `is_infinite()` guard. `f64::INFINITY` is not NaN and is
  not negative, so the function reaches the insert and returns `Ok(result)` with infinity in the
  map. `unwrap_err()` on `Ok(...)` panics. The tester documented this in `TESTER_REPORT.md`
  ("Passed: 20, Failed: 1") and correctly identified the root cause, but shipped the test without
  the implementation fix. CI will fail on this test.
- Severity: HIGH
- Action: Fix `parse_wasm_edge_weights` in `bindings/sdivi-wasm/src/weight_keys.rs`: add an
  `is_infinite()` check immediately after the `is_nan()` block at line 28 —
  `if weight.is_infinite() { return Err(format!("edge weight for key \"{key}\" is infinite; weights must be finite")); }`.
  The test is correct and must not be changed. The function's own doc comment already promises
  "weights must be finite", so this is a straightforward implementation fix.

---

#### COVERAGE: Epsilon assertion is vacuous for the `boundary_violation_delta` (i64) dimension
- File: `crates/sdivi-core/tests/prop_thresholds.rs:293`
- Issue: `prop_breach_equals_delta_gt_limit_plus_epsilon_boundary_violation` asserts
  `r.breached == (delta_f > limit + THRESHOLD_EPSILON)` where `THRESHOLD_EPSILON = 1e-9` and
  `delta_f` is an `i64` cast to `f64`. The minimum gap between consecutive integer-cast deltas
  (1.0) is nine orders of magnitude larger than epsilon, so epsilon can never flip the comparison
  outcome. The assertion is functionally identical to `delta_f > limit`. The doc comment
  acknowledges this explicitly ("epsilon has no functional effect for integer-cast deltas") and
  frames the test as a guard against a future refactor that drops the epsilon term entirely.
  The code path is exercised; only the epsilon portion of the assertion is vacuous.
- Severity: LOW
- Action: No change required. The acknowledged rationale is sound. If `boundary_violation_delta`
  is ever changed to `f64`, the test gains real epsilon coverage automatically.

---

#### NAMING: `handles_colon_in_node_id` asserts count only, not key content
- File: `bindings/sdivi-wasm/src/weight_keys.rs:108`
- Issue: The test name implies verification of correct colon-in-node-id handling, but the body
  only asserts `result.len() == 1`. The key-content assertion for this scenario is fully covered
  by the tester-added `colon_in_node_id_produces_correct_nul_key` at line 136. The count-only
  assertion adds no signal not already provided by the content test. This is a coder-authored
  test; the tester did not weaken it.
- Severity: LOW
- Action: Consider merging `handles_colon_in_node_id` into `colon_in_node_id_produces_correct_nul_key`
  (the content test already implicitly verifies a single entry via `contains_key`). Not blocking.

---

### Non-Findings (rationale recorded)

**Assertion honesty (`prop_thresholds.rs`):** All nine proptest functions derive expected values
from actual `compute_thresholds_check` calls with proptest-generated inputs. The four
`prop_breach_equals_delta_gt_limit_plus_epsilon_*` tests import `THRESHOLD_EPSILON` from the
crate under test and use the same comparison formula as the implementation — this is intentional
regression-guard behavior (acknowledged in the doc comments), not a fake assertion. No sentinel
magic numbers unrelated to implementation logic were found.

**Test weakening:** The tester added four proptest functions to `prop_thresholds.rs` and five unit
tests to `weight_keys.rs` without modifying any assertion in any pre-existing test. No weakening
occurred.

**Scope alignment:** All imports in both files resolve against the current codebase.
`compute_thresholds_check` and `THRESHOLD_EPSILON` are re-exported from
`crates/sdivi-core/src/compute/thresholds.rs`. `edge_weight_key` is exported from
`crates/sdivi-core/src/input/edge_weight.rs`. The deleted file `.tekhton/test_dedup.fingerprint`
is not imported by any test under audit.

**Implementation exercise:** All tests call real functions with un-mocked inputs. No mock
infrastructure was introduced.

**Test isolation:** All tests construct fixture data inline from literals or proptest strategies.
No test reads `.tekhton/` pipeline reports, build artifacts, snapshot files, or any mutable
project state.

**Rubric table:**

| Criterion              | prop_thresholds.rs                        | weight_keys.rs tests                               |
|------------------------|-------------------------------------------|----------------------------------------------------|
| Assertion Honesty      | PASS                                      | PASS                                               |
| Edge Case Coverage     | PASS                                      | PASS (empty map, 0.0, colon-in-ID, value fidelity) |
| Implementation Exercise| PASS                                      | PASS                                               |
| Test Weakening         | PASS — additions only                     | PASS — additions only                              |
| Test Naming            | PASS                                      | LOW concern on `handles_colon_in_node_id`          |
| Scope Alignment        | PASS                                      | HIGH — `rejects_positive_infinity_weight` asserts rejected behavior the impl does not enforce |
| Test Isolation         | PASS                                      | PASS                                               |
