## Test Audit Report

### Audit Summary
Tests audited: 4 files, 28 test functions
(5 in `crates/sdi-detection/tests/leiden_id_collision.rs`;
 6 in `crates/sdi-config/src/thresholds.rs` — 3 pre-existing, 3 new;
 13 in `crates/sdi-core/tests/validate_node_id.rs`;
 10 in `crates/sdi-core/tests/compute_thresholds_check.rs`)
Verdict: PASS

No HIGH findings. One MEDIUM and three LOW findings logged for human attention.

---

### Findings

#### COVERAGE: Test claims to verify clock-independence but cannot in M08
- File: `crates/sdi-core/tests/compute_thresholds_check.rs:121`
- Issue: `expired_today_date_consistent` carries the comment "the key invariant: the result is
  deterministic on cfg.today with no wall-clock dependency." Reading the implementation
  (`compute/thresholds.rs`), `compute_thresholds_check` ignores both `cfg.today` and
  `cfg.overrides` entirely in M08 — it reads only the four base rates. You could inject a
  hidden `SystemTime::now()` call into the implementation and this test would still pass, because
  the assertion (breach at base rate 2.0) is satisfied regardless of any clock access. The test
  cannot verify clock-independence through the override path until M09 wires per-category override
  evaluation.
- Severity: MEDIUM
- Action: Update the leading comment to accurately describe what is verified: "Documents that the
  aggregate breach check uses base rates, not per-category overrides, in M08. Clock-independence of
  the override expiry path should be tested in M09 once override wiring is complete."

#### NAMING: Test name contradicts its assertions
- File: `crates/sdi-core/tests/compute_thresholds_check.rs:107`
- Issue: `override_applied_when_not_expired` implies the unexpired override IS applied (i.e., the
  raised limit of 50.0 would suppress the breach). The assertions prove the opposite: breach occurs
  at the base rate 2.0, with the message "M08: base rate 2.0 applies; override not yet wired." The
  test body is correct; the name is misleading to future readers.
- Severity: LOW
- Action: Rename to `unexpired_override_not_yet_wired_in_m08` or
  `base_rate_applies_when_override_not_wired`.

#### COVERAGE: Weak substring assertion for trailing-slash rejection
- File: `crates/sdi-core/tests/validate_node_id.rs:33`
- Issue: `rejects_trailing_slash` asserts `format!("{err}").contains("/")`. The actual error reason
  is "must not end with '/'", which contains `/`, so the assertion passes. However, `/` appears in
  many potential error messages (path echoes, other separator messages) and would pass even if the
  wrong error variant were returned. All sibling rejection tests use more distinctive substrings
  (`"empty"`, `"./"`, `"forward"`, `".."`, `"absolute"`).
- Severity: LOW
- Action: Tighten to `contains("end with")` or `contains("trailing")` to match the actual reason
  string and guard against accidental false-positives on future refactors.

#### COVERAGE: Worst-case Leiden test missing per-node range assertion
- File: `crates/sdi-detection/tests/leiden_id_collision.rs:111`
- Issue: `leiden_all_nodes_in_community_zero_no_underflow` (n=8) describes itself as the
  "worst-case" trigger. The three other warm-start regression tests all include a loop asserting
  `comm < k` for every node. This test stops at `assert!(partition.community_count() >= 1)` — it
  would pass even if the algorithm silently produced out-of-range IDs instead of panicking.
- Severity: LOW
- Action: Add the per-node range check consistent with sibling tests:
  ```rust
  let k = partition.community_count();
  for (&node, &comm) in &partition.assignments {
      assert!(comm < k, "node {node}: community {comm} out of range [0, {k})");
  }
  ```

---

### Detailed Rubric Results

#### Assertion Honesty — PASS
All assertions test real behavior derived from actual function calls. No hardcoded magic values
disconnected from implementation logic.

- Leiden: `partition.assignments.len() == N` is derived from the explicit node count. The
  `comm < k` invariant (where `k = community_count()`) correctly captures the post-`renumber()`
  denseness guarantee.
- Thresholds (config): `override_present(&table)` traverses the real `toml::Table` instead of
  asserting a literal. Date strings passed to `validate_and_prune_overrides` are the same strings
  used as `today` — no hard-coded expected values.
- `validate_node_id`: Each acceptance/rejection decision maps directly to a rule in the
  implementation. Error message substrings are present in the actual reason strings (modulo the
  trailing-slash case flagged above).
- `compute_thresholds_check`: The base rate 2.0 used in `assert!((r.breaches[0].limit - 2.0).abs() < 1e-10)` matches `ThresholdsInput::default()::pattern_entropy_rate`. The at-limit test
  (`entropy_at_limit_not_breached`, `Some(2.0)`) correctly exercises the strict `>` (not `>=`)
  predicate in the implementation.

#### Edge Case Coverage — PASS (with LOW note above)
- Leiden: singletons with ID collision, three-node multi-member community (primary underflow
  trigger), all-in-community-zero (8 nodes), multi-iteration with renumbered partitions, cold start.
- Config thresholds: expires-today kept, expires-yesterday pruned, expires-tomorrow kept. Error
  paths (missing `expires`, non-string `expires`) covered elsewhere.
- `validate_node_id`: empty, leading `./`, trailing `/`, backslash, `..` components (leading and
  embedded), absolute path, dot in filename, standalone `.`, embedded `.` component. The last two
  explicitly document current permissive behavior as a stable contract.
- `compute_thresholds_check`: null summary (first-snapshot), each dimension individually, exact
  limit boundary, negative deltas, all four dimensions breaching simultaneously, expired override,
  unexpired override (M08 base-rate behavior), and determinism w.r.t. `cfg.today` (see MEDIUM
  finding above).

#### Implementation Exercise — PASS
All tests call real implementation entry points with real fixture data. No mock of any internal.
- Leiden: `run_leiden` is called with real `DependencyGraph` values built via
  `build_dependency_graph_from_edges`.
- Config thresholds: `validate_and_prune_overrides` is called directly with `toml::Table` built
  via `toml::from_str`.
- `validate_node_id`: `sdi_core::input::validate_node_id` is called directly.
- `compute_thresholds_check`: `sdi_core::compute::thresholds::compute_thresholds_check` is called
  directly; `ThresholdsInput` and `DivergenceSummary` are constructed from real struct literals.

#### Test Weakening — PASS (N/A for new tests)
All four files under audit contain exclusively new test functions or new additions to pre-existing
inline modules. No pre-existing assertion was broadened, removed, or replaced with a weaker form.
The three pre-existing `validate_date_format_*` tests in `thresholds.rs` are unchanged.

#### Test Naming — PASS (with LOW note above)
All names except `override_applied_when_not_expired` encode scenario and expected outcome. See
NAMING finding above.

#### Scope Alignment — PASS
All imports resolve to current public exports. No orphaned references.
- `sdi_detection::leiden::run_leiden` — `pub` in `leiden/mod.rs`, confirmed present.
- `sdi_detection::partition::{LeidenConfig, LeidenPartition}` — both `pub`, confirmed present.
- `sdi_graph::dependency_graph::build_dependency_graph_from_edges` — `pub`, no feature gate.
- `sdi_core::input::validate_node_id` — the deleted `input.rs` was replaced by `input/mod.rs`;
  the public re-export is unchanged and the import remains valid.
- `sdi_core::compute::thresholds::compute_thresholds_check` — confirmed present.
- `sdi_snapshot::delta::{DivergenceSummary, null_summary}` — both confirmed present.
- `chrono::NaiveDate` — `chrono` is a workspace dep; `sdi-core` uses it for `ThresholdsInput.today`.
- Deleted files (`crates/sdi-core/src/input.rs`, `crates/sdi-core/src/pipeline.rs`,
  `.tekhton/test_dedup.fingerprint`) are not referenced by any audited test.

#### Test Isolation — PASS
All audited tests construct fixtures entirely in memory or via `toml::from_str`. No test reads
`.tekhton/` reports, CI logs, pipeline state files, build artifacts, or any file whose content
could be mutated by a prior pipeline run. All tests are unconditionally reproducible.

---

### Additional Note: Test Duplication (not a finding)
Several happy-path tests in `crates/sdi-core/tests/compute_thresholds_check.rs`
(`null_summary_never_breaches`, `entropy_breach_detected`, `coupling_breach_detected`,
`boundary_violation_breach_detected`, `negative_delta_never_breaches`,
`multiple_breaches_all_reported`) duplicate tests already present in the inline `#[cfg(test)]`
block inside `compute/thresholds.rs`. The duplication provides no additional coverage but causes
no integrity issue. It is likely a pre-existing artefact from M08 coder output rather than tester
introduction. Not raised as a finding; flagged here for cleanup if desired.
