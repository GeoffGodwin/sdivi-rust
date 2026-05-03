## Test Audit Report

### Audit Summary
Tests audited: 2 files, 16 test functions (14 unit, 2 property)
Verdict: PASS

---

### Findings

#### COVERAGE: proptest `prop_violation_count_equals_violations_len` never exercises violation logic
- File: `crates/sdivi-core/tests/boundary_violations_proptest.rs:26`
- Issue: All generated nodes use the path prefix `crates/a/f{i}.rs`.
  The spec declares two boundaries — `crates/api/**` and `crates/db/**`.
  The glob `crates/api/**` requires the literal segment `api`; the path
  prefix `crates/a/` does not satisfy it.  Every generated node is
  therefore unscoped, `violations` is always empty, and the assertion
  reduces to `prop_assert_eq!(0usize, 0)` on every execution.
  Additionally, even if violations were generated the property
  `violation_count == violations.len()` is trivially guaranteed by the
  implementation (`boundaries.rs:274`:
  `let violation_count = violations.len() as u32;`), so the test cannot
  detect a future regression where those two fields diverge via an
  independent calculation path.
- Severity: MEDIUM
- Action: Replace the node-name template with paths that actually match
  the spec — e.g. `crates/api/f{i}.rs` for even-indexed nodes and
  `crates/db/f{i}.rs` for odd-indexed nodes.  This makes violations
  genuinely occur and exercises the property non-trivially.

#### COVERAGE: No test for `AnalysisError::InvalidConfig` (malformed glob)
- File: `crates/sdivi-core/tests/compute_boundary_violations.rs` (absent)
- Issue: `compute_boundary_violations` returns
  `AnalysisError::InvalidConfig` when a boundary's `modules` list
  contains an invalid glob pattern (e.g. `[`).  The current test suite
  has no test exercising this error path; the `Err(InvalidConfig)` branch
  in `violation.rs:compile_boundaries` is never reached.
- Severity: LOW
- Action: Add one test passing a boundary with an invalid glob string
  (`"["` is sufficient) and asserting the return value matches
  `Err(AnalysisError::InvalidConfig { .. })`.

#### COVERAGE: No test for `AnalysisError::InvalidNodeId`
- File: `crates/sdivi-core/tests/compute_boundary_violations.rs` (absent)
- Issue: `compute_boundary_violations` validates every node ID via
  `validate_node_id` before proceeding (`boundaries.rs:225-227`).  No
  test exercises the early-exit `Err(InvalidNodeId)` path.
- Severity: LOW
- Action: Add one test with a node whose ID fails `validate_node_id`
  (check what that function rejects — empty string or embedded NUL) and
  assert `Err(AnalysisError::InvalidNodeId)` is returned.

#### COVERAGE: No test for edges referencing nodes absent from `graph.nodes`
- File: `crates/sdivi-core/tests/compute_boundary_violations.rs` (absent)
- Issue: The implementation builds `node_boundaries` from `graph.nodes`
  only.  An edge whose `source` or `target` is absent from `graph.nodes`
  resolves to `None` in `node_boundaries.get()` and is silently skipped
  (`boundaries.rs:254-261`).  No test documents or asserts this contract,
  leaving callers without a verified guarantee for inconsistent inputs.
- Severity: LOW
- Action: Add one test with an edge whose `target` ID is absent from
  `graph.nodes` and assert `violation_count == 0`.

---

### Non-Findings (rationale recorded)

- **Assertion honesty (unit tests):** All 14 unit-test assertions derive
  their expected values from the documented algorithm (boundary assignment
  rules, allow-list membership, sorted output, duplicate-edge counting).
  No hard-coded magic numbers unrelated to implementation logic were found.
- **Test weakening:** The tester added two new tests
  (`duplicate_edge_produces_two_violations`,
  `duplicate_same_boundary_edge_produces_no_violation`) without modifying
  any assertion in the 12 tests written by the coder.  No weakening
  occurred.
- **Scope alignment:** All imports (`compute_boundary_violations`,
  `BoundaryDefInput`, `BoundarySpecInput`, `DependencyGraphInput`,
  `EdgeInput`, `NodeInput`) are re-exported from `sdivi_core::lib.rs`
  and match the current public surface.  No orphaned references.
  `proptest` is listed in `[dev-dependencies]` of `sdivi-core/Cargo.toml`.
- **Naming:** All 16 test names encode both scenario and expected outcome.
- **Implementation exercise:** Every test calls
  `compute_boundary_violations` with real, un-mocked inputs.
- **Test isolation:** All fixture data is constructed inline in memory.
  No test reads mutable project files, pipeline artifacts, or
  `.tekhton/` reports.
- **`prop_violations_are_sorted`:** Uses `crates/a/**` and `crates/b/**`
  against nodes with matching prefixes; violations are genuinely produced
  and the sorted-output invariant is meaningfully exercised.
