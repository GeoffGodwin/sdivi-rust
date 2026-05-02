# Coder Summary
## Status: COMPLETE

## What Was Implemented

- **`LeidenGraph` self-loops support** (`graph.rs`): Added `self_loops: Vec<f64>` field.
  Changed `from_edges_weighted` to accumulate self-loops instead of dropping them.
  Updated `degree[u]` to include `2 × self_loops[u]` (standard undirected convention).
  Updated `total_weight` to use `cross_edge_total + self_loop_total` (not `degree.sum()/2`
  which would double-count self-loops). Fixed `edge_weight(u, u)` to return `self_loops[u]`.
  Added inline unit tests (5 tests).

- **`aggregate_network` correctness fix** (`aggregate.rs`): Rewrote the inner edge
  traversal to walk only the upper triangle (`v >= u`), visiting each undirected
  cross-edge exactly once (fixes prior 2× double-count). Intra-community cross-edges
  now emit self-loops on the aggregate super-node. Source node self-loops propagate to
  the aggregate super-node regardless of partition.

- **`ModularityState` self-loop awareness** (`modularity.rs`): `from_assignment` adds
  `graph.self_loops[i]` to `inner_edges[c]` for each node (self-loops are always
  internal to a node's own community). `remove_node` subtracts `graph.self_loops[node]`
  from the old community's `inner_edges` and sets the singleton's `inner_edges[node]`
  to `graph.self_loops[node]`. `add_node` clears the singleton slot and adds
  `graph.self_loops[node]` to the new community.

- **`compute_modularity` and `compute_stability`** (`quality.rs`): Both now add
  `graph.self_loops[i]` to `inner[c]` per node, correctly counting self-loop weight
  as internal community weight.

- **Test plumbing** (`lib.rs`): Added `#[doc(hidden)] pub mod internal` re-exporting
  `LeidenGraph`, `AggregateResult`, `aggregate_network`, and `compute_modularity` for
  integration test access.

- **New integration test file** (`tests/aggregate_invariance.rs`): Four hand-derived
  test cases verifying aggregate structure (self-loops, no double-counting) plus
  modularity invariance. One proptest property test with 256 cases.

- **CHANGELOG.md** updated with M17 entry.

## Root Cause (bugs only)

Two correctness bugs in the Leiden recursive machinery:
1. **`aggregate_network` dropped intra-community edges** — the `std::cmp::Ordering::Equal`
   arm was a no-op, so any cross-edge within a community was silently discarded. These
   edges must become self-loops on the aggregate super-node to preserve the strong
   intra-community pull that modularity relies on.
2. **`aggregate_network` double-counted inter-community edges** — the code iterated
   `graph.adj[u]` for all u, visiting each undirected edge twice (once as (u,v) from u,
   once as (v,u) from v). The fix: skip `v < u` in the inner loop (upper-triangle walk).

Together these caused every graph to collapse to a single community after one
aggregation step because the aggregate graph had zero edge weight, making the
"merge all" partition optimal by default.

## Files Modified

- `crates/sdivi-detection/src/leiden/graph.rs` (MODIFIED) — self-loops support
- `crates/sdivi-detection/src/leiden/aggregate.rs` (MODIFIED) — double-count fix + self-loops
- `crates/sdivi-detection/src/leiden/modularity.rs` (MODIFIED) — self-loop awareness
- `crates/sdivi-detection/src/leiden/quality.rs` (MODIFIED) — self-loop awareness
- `crates/sdivi-detection/src/leiden/mod.rs` (MODIFIED) — module visibility for re-exports
- `crates/sdivi-detection/src/lib.rs` (MODIFIED) — internal test helpers
- `crates/sdivi-detection/tests/aggregate_invariance.rs` (NEW) — 4 hand-derived + 1 proptest
- `CHANGELOG.md` (MODIFIED)

## Human Notes Status

No explicit Human Notes section in the milestone — all deliverables listed under
Deliverables and Tests sections. All addressed.

## Architecture Change Proposals

**Adding `pub mod internal` to expose test plumbing**
- **Current constraint**: `LeidenGraph`, `aggregate_network`, and `compute_modularity`
  are `pub(crate)` — not accessible from integration tests in `tests/`
- **What triggered this**: Milestone requires `tests/aggregate_invariance.rs` that
  directly tests `compute_modularity` and `aggregate_network` invariance properties,
  but these are internal types not reachable from integration tests otherwise
- **Proposed change**: Added `#[doc(hidden)] pub mod internal` to `lib.rs` re-exporting
  `LeidenGraph`, `aggregate_network`, `AggregateResult`, and `compute_modularity`.
  Changed the items from `pub(crate)` to `pub` with `#[doc(hidden)]`. This is a
  standard Rust pattern for test plumbing (used by serde, syn, etc.).
- **Backward compatible**: Yes — no existing public items changed or removed
- **ARCHITECTURE.md update needed**: No — test plumbing is not architectural

## Docs Updated

None — all changed types/functions are `#[doc(hidden)]` and not part of the stable
user-facing API. `CHANGELOG.md` updated with the M17 bug-fix entry.

## Files Modified (auto-detected)
- `.claude/milestones/MANIFEST.cfg`
- `.claude/milestones/m17-leiden-self-loops-and-aggregate-fix.md`
- `.tekhton/CODER_SUMMARY.md`
- `.tekhton/DRIFT_LOG.md`
- `.tekhton/JR_CODER_SUMMARY.md`
- `.tekhton/PREFLIGHT_REPORT.md`
- `.tekhton/REVIEWER_REPORT.md`
- `.tekhton/TESTER_REPORT.md`
- `.tekhton/test_dedup.fingerprint`
- `CHANGELOG.md`
- `crates/sdivi-detection/src/leiden/aggregate.rs`
- `crates/sdivi-detection/src/leiden/graph.rs`
- `crates/sdivi-detection/src/leiden/mod.rs`
- `crates/sdivi-detection/src/leiden/modularity.rs`
- `crates/sdivi-detection/src/leiden/quality.rs`
- `crates/sdivi-detection/src/lib.rs`
