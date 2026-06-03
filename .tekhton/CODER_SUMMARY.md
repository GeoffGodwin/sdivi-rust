# Coder Summary

**Agent:** coder
**Date:** 2026-06-02
**Branch:** main

---

## Status: COMPLETE

## What Was Implemented

**M49.1: Leiden Hang — Capture the Minimal Repro and Contain it in CI**

1. **Enabled proptest `fork` + `timeout` features** for `sdivi-detection`'s dev-dependency by adding `features = ["fork", "timeout"]` to `crates/sdivi-detection/Cargo.toml`. Without these features, `PROPTEST_TIMEOUT` and `ProptestConfig { fork: true, timeout: … }` are silently ignored — the existing bug that caused CI hangs.

2. **Split the `proptest!` block in `refinement.rs`** into two separate blocks:
   - Block 1 (no change): `prop_refine_does_not_increase_coarse_communities` — uses `ProptestConfig::with_cases(256)`, no fork overhead needed.
   - Block 2 (new): `prop_refine_modularity_does_not_decrease` — uses `ProptestConfig { timeout: 10_000, fork: true, ..ProptestConfig::with_cases(256) }`. Each case now runs in a forked subprocess; a non-converging `run_leiden` call is killed after 10 seconds and reported as a test failure with its minimized input, rather than hanging the test binary.

3. **Captured the minimal hanging case** by running `PROPTEST_CASES=4096` with the new fork+timeout config. Proptest found and minimized to:
   - **n=6, K_{1,5} star graph** (node 3 connected to all of 0,1,2,4,5), **seed=0**
   - Stored as `cc 7bcb943d7e48407056966a9ca32d5f7f276d3d9e694db514ac5de978d843f27e` in `refinement.proptest-regressions`
   - Replay: `PROPTEST_CASES=1 cargo test … prop_refine_modularity_does_not_decrease` fails in ~40s (never hangs)

4. **Created `leiden_termination.rs`** (new test file) with:
   - `leiden_with_corrected_refine_gives_positive_modularity` — full-Leiden integration test (moved from `refinement.rs` to make room under the 300-line ceiling)
   - `leiden_termination_regression_star_n6_seed0` — `#[ignore = "fails until M49.2 fixes leiden blowup; see milestone"]` deterministic regression test. Constructs the minimal case on a thread with a 30-second join timeout; asserts `run_leiden` returns before the deadline. M49.2 un-ignores this once the non-termination bug is fixed.

## Root Cause (bugs only)

`prop_refine_modularity_does_not_decrease` calls `run_leiden` on random `(n, edges, seed)` inputs with `n ≤ 8`. For specific inputs (e.g. K_{1,5} star with seed=0), the Leiden algorithm takes far longer than typical cases: the outer loop in `leiden_recursive` can run many iterations each spawning a recursive descent on the aggregated graph, producing exponential work for pathological partition sequences. Without `fork: true` + `timeout: 10_000`, proptest could not time-out the hanging subprocess — `PROPTEST_TIMEOUT` is silently ignored without the Cargo features. The test binary hung until the 30-minute CI job timeout killed the runner.

## Files Modified

- `crates/sdivi-detection/Cargo.toml` — added `features = ["fork", "timeout"]` to proptest dev-dep
- `crates/sdivi-detection/tests/refinement.rs` — split `proptest!` block; added fork+timeout ProptestConfig to the modularity test; moved `leiden_with_corrected_refine_gives_positive_modularity` to `leiden_termination.rs` (line count compliance)
- `crates/sdivi-detection/tests/leiden_termination.rs` (NEW) — full-Leiden integration test + ignored M49.1 regression guard
- `crates/sdivi-detection/tests/refinement.proptest-regressions` (NEW) — committed minimal hanging case for immediate regression replay on CI

## Docs Updated

None — no public-surface changes in this task (test-only and dev-dependency-only changes).

## Human Notes Status

No Human Notes section present in this task.

## Observed Issues (out of scope)

- `crates/sdivi-detection/tests/renumber_delegation.rs:83,85` — two pre-existing `clippy::iter_cloned_collect` warnings (`iter().copied().collect()` should be `.to_vec()`). Not caused by this task's changes; unmodified file.
