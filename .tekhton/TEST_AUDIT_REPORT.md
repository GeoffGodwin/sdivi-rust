## Test Audit Report

### Audit Summary
Tests audited: 2 files, 12 test functions
Verdict: PASS

---

### Findings

#### COVERAGE: Incomplete error assertion at exact compression-ratio boundary
- File: `crates/sdivi-config/tests/validate_boundaries.rs:23`
- Issue: `validate_boundaries_rejects_compression_ratio_at_1_0` matches
  `ConfigError::InvalidValue { key, .. }` and asserts only the `key` field;
  the `message` field is silently discarded via `..`.  The structurally identical
  test for the `1.5` case (line 44) asserts both `key` and `message` (checking
  that the supplied value is echoed back).  The discrepancy leaves the
  exact-boundary error message unverified.  The implementation at `load.rs:192`
  produces `format!("must be in [0.0, 1.0), got {r}")`, so a
  `message.contains("1.0")` check is straightforward to add.
- Severity: LOW
- Action: Expand the match arm on line 33 to also bind and assert `message`:
  ```rust
  ConfigError::InvalidValue { key, message } => {
      assert_eq!(key, "boundaries.leiden_min_compression_ratio");
      assert!(message.contains("1.0"),
              "error message must echo the supplied value, got: {message}");
  }
  ```

#### COVERAGE: Soft conditional assertion in behavioral-difference test
- File: `crates/sdivi-detection/tests/leiden_depth_cap.rs:139`
- Issue: `depth_cap_at_1_produces_valid_but_different_partition_than_uncapped`
  computes both a depth-1-capped and an uncapped partition, then enters an
  `if !differ { eprintln!(...) }` branch where — if the two partitions
  coincidentally agree — the test **makes zero assertions and passes
  unconditionally**.  Because behavior is fully deterministic for a fixed seed
  and graph, this branch is either always taken or never taken; no test run can
  tell which.  The test name claims to verify a behavioral difference (i.e.,
  that the depth cap returns an early-exit value rather than the fully converged
  one), but enforcement is optional.  The other three tests in the file verify
  *structural validity* (all nodes covered, community IDs in range), not the
  *behavioral change* caused by the cap firing.
- Severity: MEDIUM
- Action: One of two clean fixes:
  (a) **Assert unconditionally** — run both variants locally on this seed+graph
  to confirm they differ (the 25-node ring-of-5-cliques with `seed=42` should
  compress enough to trigger at least one recursive call), then replace
  `if !differ { eprintln!(...) }` with
  `assert!(differ, "depth-1 cap must produce a different partition than depth-32")`.
  (b) **Remove the comparison test** — the three validity tests already exercise
  the code path.  Delete this function to avoid the false impression of
  behavioral coverage it does not deliver.

#### NAMING: Test name implies depth cap fires; fixture prevents recursion
- File: `crates/sdivi-detection/tests/leiden_depth_cap.rs:183`
- Issue: `depth_cap_at_depth_1_is_minimum_working_configuration` uses an
  isolated-nodes graph (no edges).  The test body comment correctly states
  "The depth cap does NOT fire here because recursion requires compression."
  On an edgeless graph, `local_move_phase` makes no moves on the first
  iteration, so the outer loop exits immediately via `!moved`, never reaching
  the aggregation step that triggers recursion.  The name "minimum working
  configuration" implies the cap fires at its configured limit, when the test
  actually verifies that Leiden exits cleanly on a trivially structured input
  regardless of how the depth cap is set.
- Severity: LOW
- Action: Rename to `depth_cap_setting_of_1_does_not_prevent_trivial_convergence`
  (or similar) and adjust the docstring to describe what is actually being
  exercised — Leiden's graceful handling of edgeless graphs rather than the
  depth-cap guard.

---

### Scope and Isolation Notes

**No orphaned imports or stale symbol references detected.**  Both test files
reference symbols present in the current codebase: `load_with_paths`,
`ConfigError` (`sdivi-config`); `run_leiden`, `LeidenConfig`
(`sdivi-detection`); `build_dependency_graph_from_edges` (`sdivi-graph`).
None of the symbols deleted this milestone (`.tekhton/JR_CODER_SUMMARY.md`,
`.tekhton/test_dedup.fingerprint`) are imported or referenced by any test
under audit.

**Isolation is sound.**  `validate_boundaries.rs` writes all TOML content to
`tempfile::NamedTempFile` and passes the temp path to
`load_with_paths(Some(f.path()), None)`.  Both the global-config path
argument and the env-var layer (`apply_env_overrides`) are bypassed; no
machine-local sdivi config can influence the outcome.  `leiden_depth_cap.rs`
constructs all graphs from inline literal edge lists and reads no project
files.  Neither file's pass/fail outcome depends on build artifacts, pipeline
state, or any mutable file in the repository.

**Assertion honesty.**  All asserted constants are derived from the
implementation:
- The defaults `0.1` and `32` match `BoundariesConfig::default()` at
  `config.rs:113-116`.
- The node-coverage counts (12, 25, 5) match the fixture-construction
  logic in the same test file.
- The community-ID validity assertion (`comm < community_count`) is sound:
  `compute_stability` in `quality.rs:7-43` skips empty communities; after
  `renumber` in `leiden_recursive`, all community IDs lie in `[0, k)` where
  `k == stability.len() == community_count()`.
- `initial_assignment_from_cache(None, n)` returns `(0..n).collect()` (each
  node in its own singleton), confirming the 5-community assertion in the
  isolated-nodes test.

No test asserts a hard-coded value that is not derivable from the implementation.
