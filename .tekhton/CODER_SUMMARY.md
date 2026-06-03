# Coder Summary

**Agent:** coder
**Date:** 2026-06-03
**Branch:** main

---

## Status: COMPLETE

## What Was Implemented

**M49.2: Leiden — Fix the `O(max_iterations^depth)` Recursion Blowup**

1. **Restructured `leiden_recursive` in `crates/sdivi-detection/src/leiden/mod.rs`.**
   The old loop body called `leiden_recursive` recursively on every iteration where
   `moved == true`, producing O(max_iter^depth) total calls.  The new structure
   separates the function into two phases:
   - **Phase 1:** Run local moves to convergence (`for _iter in 0..max_iter` with break
     on `!moved`). If no moves fire, return immediately — already locally optimal.
   - **Phase 2:** After convergence, refine + aggregate + recurse **once**. Flatten and
     renumber; return.
   
   Total recursive calls per invocation: at most 1 (plus the depth-cap guard at
   `depth >= max_recursion_depth`). Total work: O(max_iter × n × max_recursion_depth)
   — polynomial.

2. **Un-ignored the M49.1 regression test** in `leiden_termination.rs`:
   removed `#[ignore = "fails until M49.2 fixes leiden blowup; see milestone"]` from
   `leiden_termination_regression_star_n6_seed0`. Updated the doc comment to note that
   M49.2's restructure eliminates the blowup.

3. **Added brute-force termination sweep test** `termination_sweep_known_graphs_all_seeds`
   in `leiden_termination.rs`. Sweeps 5 graphs (K_{1,5} star, K4, path-n6, two-K3-bridge,
   K5) × all seeds 0..=255 (1280 `run_leiden` calls total). Asserts every call returns a
   valid partition covering all nodes. Completes in ~1.7s in debug builds.

4. **Fixed pre-existing version mismatch** in `bindings/sdivi-wasm/pkg-template/package.json`:
   bumped from `"0.2.48"` to `"0.2.49"` to match the workspace Cargo.toml (the M49.1
   run bumped Cargo.toml but not package.json, causing `wasm_package_json_version_matches_workspace`
   to fail in `cargo test --workspace`).

5. **Updated CHANGELOG.md** (Unreleased section) documenting the non-termination fix and
   the partition-value change.

6. **Updated MIGRATION_NOTES.md** with an M49.2 entry explaining the RNG sequence change
   and its impact on community assignments, with an escape-hatch `[thresholds.overrides]`
   snippet.

## Root Cause (bugs only)

`leiden_recursive` called the recursive descent (`leiden_recursive(&agg_graph, ..., depth+1)`)
**inside** the `for _iter in 0..max_iter` loop, on every iteration where
`local_move_phase` returned `true`.  For inputs where local moves fire on consecutive
iterations (e.g. K_{1,5} star, n=6, seed=0), this caused up to `max_iter` recursive
descents per level, each of which could do `max_iter` more, compounding to
O(max_iter^depth) total work.  For max_iter=100 even at depth 2, this is 10,000
recursive calls; at depth 8, ~10^16.  The M28 recursion-depth cap bounded depth but
not the number of descents per level.

Fix: separate Phase 1 (local-move convergence loop) from Phase 2 (single refine +
aggregate + recurse).  The recursive call now happens at most once per invocation.

## Files Modified

- `crates/sdivi-detection/src/leiden/mod.rs` — restructured `leiden_recursive` to
  eliminate the O(max_iter^depth) recursion blowup; added doc comment explaining the fix
- `crates/sdivi-detection/tests/leiden_termination.rs` — un-ignored the M49.1 regression
  guard; updated its doc comment; added `termination_sweep_known_graphs_all_seeds`
- `bindings/sdivi-wasm/pkg-template/package.json` — bumped version 0.2.48 → 0.2.49
  (pre-existing mismatch from M49.1)
- `CHANGELOG.md` — added Unreleased entry for the non-termination fix
- `MIGRATION_NOTES.md` — added M49.2 section for partition-assignment migration guidance

## Docs Updated

- `CHANGELOG.md` — Unreleased section: describes the blowup fix and partition-value change
- `MIGRATION_NOTES.md` — new M49.2 entry with migration action and escape-hatch snippet

## Human Notes Status

No Human Notes section present in this task.

## Observed Issues (out of scope)

- `crates/sdivi-detection/tests/renumber_delegation.rs:83,85` — two pre-existing
  `clippy::iter_cloned_collect` warnings (pre-existing from M49.1, noted by the reviewer;
  unrelated to M49.2).
