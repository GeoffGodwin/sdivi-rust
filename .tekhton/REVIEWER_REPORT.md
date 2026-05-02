# Reviewer Report — M17: Leiden Self-Loops and Aggregate Fix

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `graph.rs:172` — `#[allow(dead_code)]` on a `pub fn` is superfluous: Rust's `dead_code` lint does not fire on public items (assumed reachable externally). The annotation is harmless but misleading; `#[allow(unused)]` would not fire either. Can be dropped.
- `quality.rs:compute_stability` — with self-loops, `inner[c]` can exceed `n*(n-1)/2`, making stability > 1.0 for multi-node communities. In practice this cannot occur because `build_partition` calls `compute_stability` on the original `LeidenGraph` built from a `DependencyGraph` (which has no self-loops), so `self_loops[i] == 0.0` always at the call site. The code path is inert today but semantically surprising if `compute_stability` is ever exposed for aggregate-level introspection.
- `modularity.rs:add_node` — when `to == node` (node returns to its own singleton), `sigma_tot[node]` and `size[node]` are double-incremented (they were already set by `remove_node` to `degree[node]` and `1` respectively). This is pre-existing behavior that M17 did not introduce; within a single `local_move_phase` pass each node is visited exactly once so the corrupted sigma_tot is never read a second time for that node. Noted for awareness.

## Coverage Gaps
- No test covers `run_leiden` producing multiple stable communities end-to-end after the fix (i.e., regression test confirming the "collapse to 1 community" bug is gone under the full algorithm). The aggregate invariance property tests are thorough for the structural fix, but an end-to-end sanity check (e.g., two cliques → two communities) would guard against future regressions at the `mod.rs` level.
- `compute_stability` is not tested with self-loops present; low priority given the inert call site, but worth a note.

## ACP Verdicts
- ACP: pub mod internal — ACCEPT — Standard Rust test-plumbing pattern (`#[doc(hidden)]` + explicit "not stable API" prose). Items are in `sdivi-detection`, not in `sdivi-core` (the API-stability boundary). Well-implemented: all re-exported items are `#[doc(hidden)]`, the module is documented as ephemeral, and the integration test relies on nothing beyond what is re-exported.

## Drift Observations
- `aggregate.rs:39` — `std::collections::BTreeMap` is imported via full path rather than a `use` statement at the top of the file. The rest of the codebase uses top-level `use` declarations. Cosmetic inconsistency, not a correctness issue.
- `modularity.rs:add_node` comment — "When `to == node` this is immediately overwritten by the self-loop addition below" accurately describes `inner_edges` but is silent about the sigma_tot/size double-increment on the same code path. If someone later reads this comment expecting the singleton round-trip to be fully no-op, they may be confused.
