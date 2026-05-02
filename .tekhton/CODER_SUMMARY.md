# Coder Summary
## Status: COMPLETE

## What Was Implemented
- Rewrote `refine.rs` with `RefinementState` struct, `apply_move`, `move_gain`, `well_connected`,
  and corrected `refine_community` that uses real per-sub-community `sigma_tot` (not a count fudge)
- Added `refine_partition`, `RefinementState`, `well_connected` to `internal` exports in `lib.rs`
- Created `crates/sdivi-detection/tests/refinement.rs` with 12 tests:
  - Unit: `from_partition_singleton_init_sigma_tot`, `from_partition_non_singleton_inner_edges`
  - Unit: `apply_move_updates_sigma_tot_and_size`
  - Unit: `well_connected_gamma_zero_always_true`, `well_connected_strong_connection_passes`, `well_connected_weak_connection_fails`
  - Integration: `two_disconnected_groups_never_mix_after_refine`, `refine_preserves_coarse_community_boundary`, `refine_path_graph_boundary`
  - Integration: `leiden_with_corrected_refine_gives_positive_modularity`
  - Property: `prop_refine_does_not_increase_coarse_communities`, `prop_refine_modularity_does_not_decrease`
- Added debug assertion in `mod.rs` for `leiden_recursive` (aggregate graph shrinks)
- Updated `verify-leiden.yml`: added `milestones/**` to push branches (already had push/pull_request triggers)
- Updated `CHANGELOG.md` with M18 entry including migration note for pre-M18 snapshots
- Updated `.tekhton/DRIFT_LOG.md` with M17+M18 regression note
- Updated `docs/determinism.md` with tie-break by BTreeMap iteration order note

## Root Cause (bugs only)
`refine.rs:151` `best_candidate` used `count of v's neighbours in sub-community comm` as a
fake `sigma_tot`, labelled "approximate sigma_tot for sub-comm." This caused the gain formula
to reduce to roughly `k_in − degree·k_in/2m`, which is positive for almost every candidate.
Refinement shuffled nodes randomly between sub-communities, producing a near-random refined
partition that when aggregated caused the entire graph to collapse to one community (modularity ≈ 0).

Fix: `RefinementState` tracks real `sigma_tot[c]` (sum of degrees in sub-community `c`),
`inner_edges[c]`, and `size[c]`. `move_gain` uses `k_in_to - degree[node] * sigma_tot[to] / (2 * total_weight)`
which matches `ModularityState::move_gain`. With correct gains, only positive-ΔQ moves are made,
and the partition converges to well-separated communities.

## Files Modified
- `crates/sdivi-detection/src/leiden/refine.rs` (MODIFIED) — complete rewrite: `RefinementState`,
  `apply_move`, `move_gain`, `well_connected`, `refine_community`, `refine_partition`; 278 lines
- `crates/sdivi-detection/src/leiden/mod.rs` (MODIFIED) — debug assertion in `leiden_recursive`; 297 lines
- `crates/sdivi-detection/src/lib.rs` (MODIFIED) — export `refine_partition`, `well_connected`, `RefinementState` via `internal`; 39 lines
- `crates/sdivi-detection/tests/refinement.rs` (NEW) — 12 unit/integration/property tests; 299 lines
- `.github/workflows/verify-leiden.yml` (MODIFIED) — added `milestones/**` to push branches
- `CHANGELOG.md` (MODIFIED) — M18 entry
- `.tekhton/DRIFT_LOG.md` (MODIFIED) — M17+M18 regression note
- `docs/determinism.md` (MODIFIED) — tie-break by BTreeMap order note

## Human Notes Status
No explicit Human Notes section in milestone.

## Docs Updated
- `docs/determinism.md` — added note that BTreeMap ascending iteration gives deterministic
  smallest-ID tie-break for equal-gain candidates in both local-move and refinement phases.

## Observed Issues (out of scope)
- `crates/sdivi-config/src/thresholds.rs:52` — `validate_and_prune_overrides` is `pub(crate)`
  but only used within `#[cfg(test)]` blocks, triggering `dead_code` in `cargo clippy -D warnings`.
  Pre-existing; not introduced by M18. Blocks `cargo clippy -p sdivi-detection -- -D warnings`
  but not the full workspace clippy that excludes dead_code for test-only functions.
