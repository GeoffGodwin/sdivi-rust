## Test Audit Report

### Audit Summary
Tests audited: 2 files, 11 test functions
(5 in `crates/sdi-detection/tests/leiden_id_collision.rs`;
 6 in `crates/sdi-config/src/thresholds.rs` — 3 pre-existing, 3 new)
Verdict: PASS

---

### Findings

#### COVERAGE: `leiden_all_nodes_in_community_zero_no_underflow` missing per-node validity assertion
- File: `crates/sdi-detection/tests/leiden_id_collision.rs:111`
- Issue: This test is described as the "worst-case" underflow trigger (all 8 nodes in community 0).
  The other three warm-start regression tests all include a loop that asserts `comm < k` for every
  node in the returned partition. This test stops at `assert!(partition.community_count() >= 1)`
  — it verifies the algorithm didn't panic but does not verify the output partition is valid.
  If the offset fix were to silently produce out-of-range community IDs rather than panic, this
  test would still pass while the stronger regression tests would catch it.
- Severity: LOW
- Action: Add the per-node range assertion consistent with the sibling tests:
  ```rust
  let k = partition.community_count();
  for (&node, &comm) in &partition.assignments {
      assert!(comm < k, "node {node}: community {comm} out of range [0, {k})");
  }
  ```

#### COVERAGE: `leiden_singleton_partition_with_ids_equal_to_node_indices_completes` does not exercise the underflow path
- File: `crates/sdi-detection/tests/leiden_id_collision.rs:43`
- Issue: The comment says this tests "community IDs collide with node indices," implying it
  exercises the underflow bug. It does not. Singleton communities (one member each) cannot trigger
  the underflow: `remove_node(X)` decrements `size[X]` from 1 to 0, then immediately resets it to
  1. The net effect is a no-op on `size[X]`, so the pre-fix code also passes this test. The test
  correctly validates that the offset fix does not corrupt singleton-slot logic (a regression test
  for the fix itself), but the comment overstates what the scenario proves. The primary underflow
  trigger is covered by `leiden_three_nodes_in_community_zero_no_underflow`.
- Severity: LOW
- Action: Update the test's doc comment to accurately describe its purpose: "Verifies that the
  offset fix does not corrupt singleton-slot logic when community IDs happen to equal node indices.
  Singleton communities never trigger the underflow, so this test exercises the fix's correctness
  without exercising the original bug scenario."

---

### Detailed Rubric Results

#### Assertion Honesty — PASS
All assertions test real behavior. No hardcoded magic values disconnected from implementation
logic. Specific findings:

- `partition.assignments.len() == N` — derived from the explicit node count of the fixture graph.
- `comm < k` where `k = partition.community_count()` — meaningful: `community_count()` returns
  `stability.len()`, which equals the number of non-empty communities after `compute_stability`
  and the final `renumber()` call. The assertion would catch any community-ID range corruption.
- `override_present(&table)` in `thresholds.rs` — correctly traverses the real `toml::Table`
  structure rather than asserting a literal.
- The hardcoded date `"2026-04-29"` in threshold tests is passed as a parameter to
  `validate_and_prune_overrides`, not read from the system clock. Tests are deterministic
  regardless of execution date.

#### Edge Case Coverage — PASS (with LOW note above)
Leiden tests cover: singletons with ID collision, three-node multi-member community (primary bug
trigger), all-nodes-in-one-community (worst case), multi-iteration with renumbered partitions, and
cold start. Threshold tests cover the three-way boundary: expires today (kept), expires yesterday
(pruned), expires tomorrow (kept). Error paths (missing `expires`, non-string `expires`) are
covered by the integration tests in `crates/sdi-config/tests/threshold_overrides.rs`.

#### Implementation Exercise — PASS
All Leiden tests call the real `run_leiden` entry point with real `DependencyGraph` objects built
via `build_dependency_graph_from_edges`. No mocks. Threshold tests call
`validate_and_prune_overrides` directly against real `toml::Table` fixtures created via
`toml::from_str`. No mocks.

#### Test Weakening — PASS (N/A for new tests)
All five Leiden tests are new. The three threshold boundary tests are new. The three pre-existing
`validate_date_format_*` tests are unchanged.

#### Test Naming — PASS
All function names encode both the scenario and the expected outcome:
- `leiden_three_nodes_in_community_zero_no_underflow` ✓
- `leiden_multi_iteration_no_underflow_on_two_cliques` ✓
- `expires_equal_to_today_is_kept` ✓
- `expires_one_day_before_today_is_pruned` ✓

#### Scope Alignment — PASS
All imports resolve to current public exports:
- `sdi_detection::leiden::run_leiden` — `pub` via `leiden/mod.rs`, re-exported from `lib.rs`
- `sdi_detection::partition::{LeidenConfig, LeidenPartition}` — both `pub`
- `sdi_graph::dependency_graph::build_dependency_graph_from_edges` — `pub`, always available
  (no feature gate required)
- `validate_and_prune_overrides` — `pub(crate)` in `thresholds.rs`, accessible from inline test
  module via `super::*`

The deleted file `.tekhton/test_dedup.fingerprint` is not referenced by any test.
The prior-run rename (`validate_and_prune_overrides` → `validate_overrides_format`) was
reverted before the tester ran; the function name in the tests matches the current implementation.

#### Test Isolation — PASS
All Leiden tests construct fixtures in memory (`ring_graph`, `clique_graph` helpers, inline
`LeidenPartition` structs). All threshold tests use `toml::from_str` for inline fixture data.
No test reads `.tekhton/`, CI logs, pipeline reports, build artifacts, or any mutable
project-state file.
