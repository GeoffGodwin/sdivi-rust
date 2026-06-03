# Reviewer Report

**Reviewer:** code-review agent
**Date:** 2026-06-03
**Milestone:** M49.2 — Fix Leiden O(max_iter^depth) Recursion Blowup
**Review cycle:** 1 of 4

---

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `MIGRATION_NOTES.md` escape-hatch snippet uses `[thresholds.overrides.boundary_violations]` with key `boundary_violation_rate = 999.0`. `boundary_violations` is not a pattern category name; per-category override blocks apply to named pattern categories (e.g. `error_handling`, `concurrency`). If the config parser only dispatches `pattern_entropy_rate`/`convention_drift_rate` within per-category blocks and ignores `boundary_violation_rate` there, the snippet silently does nothing and adopters get no suppression. Consider replacing with a re-baseline recommendation, or confirming that `boundary_violation_rate` is valid inside per-category override blocks (and add a test or note to that effect).
- `termination_sweep_known_graphs_all_seeds` has no per-invocation timeout. If the fix ever regresses on a seed/graph pair not in the known-regression test, CI will hang to the job timeout rather than failing with a clear message. The existing `leiden_termination_regression_star_n6_seed0` test covers the known pathological case with a 30-second thread timeout, which mitigates this for the captured minimal case.

## Coverage Gaps
- None

## Drift Observations
- `crates/sdivi-detection/tests/renumber_delegation.rs:83,85` — Pre-existing `clippy::iter_cloned_collect` warnings noted by the coder (unrelated to M49.2). Should be cleaned up in a follow-on pass so `cargo clippy -- -D warnings` remains green per CLAUDE.md Rule 20.

---

## Review Notes (informational)

### Algorithm restructuring (`leiden/mod.rs:124–202`)
The two-phase restructure is correct. Phase 1 runs local moves to convergence; `any_moved` tracks whether at least one sweep fired. The early return on `!any_moved` is semantically sound: if no move was beneficial on the first sweep the partition is already locally optimal, and refinement/aggregation would produce an identity result. Phase 2 calls `leiden_recursive` exactly once per invocation at `depth + 1`. Total work is O(max_iter × n × max_recursion_depth) — polynomial.

The identity-compression guard (`agg_graph.n >= graph.n`) and the compression-ratio cutoff together prevent infinite recursion independent of the depth cap. The `#[allow(clippy::too_many_arguments)]` annotation has a valid inline justification per CLAUDE.md lint discipline. File is 242 lines, well under ceiling.

### Tests (`leiden_termination.rs`)
Un-ignoring the M49.1 regression test is correct — the fix makes that case complete in microseconds. The 30-second thread timeout provides safe fail-fast if the case ever regresses. The brute-force sweep (5 graphs × 256 seeds = 1 280 `run_leiden` calls) with `p.assignments.len() == n` as the invariant is a solid completeness gate. 167 lines.

### `package.json` version bump
Correctly synchronized to `0.2.49`. The `wasm_package_json_version_matches_workspace` integration test would have continued to fail without this fix.

### `CHANGELOG.md`
Unreleased entry is accurate: cites the algorithm paper reference, notes the RNG draw-sequence change and `snapshot_version` stability, references `MIGRATION_NOTES.md`.

### `MIGRATION_NOTES.md`
Schema-unchanged / config-unchanged declarations are correct. Partition-value impact section accurately describes the RNG draw-order change and notes quality is preserved within the `verify-leiden` tolerance. The escape-hatch snippet concern is documented above.
