# Coder Summary
## Status: COMPLETE

## What Was Implemented

### Perf Bug 1 Fix — Induced-Subgraph Refinement
- Added `LeidenGraph::induced_subgraph` method that builds a subgraph from a
  subset of nodes, sorted ascending by global ID, returning `(LeidenGraph, local_to_global)`.
- Rewrote `refine_community` to build the induced subgraph and run `RefinementState`
  on the subgraph only (O(|members| + induced_edges) instead of O(n + total_edges)).
- Removed the `in_coarse: Vec<bool>` allocation.
- After convergence, maps local sub-community IDs to globally unique IDs using
  the smallest global member of each sub-community (prevents ID collisions across
  sibling calls to `refine_community`).

### Perf Bug 2 Fix — Compression-Ratio Recursion Cap
- Extended `leiden_recursive` signature with `min_compression_ratio: f64`,
  `max_recursion_depth: u32`, and `depth: u32`.
- Replaced identity-only break (`agg.n >= graph.n`) with a compression-ratio
  check: stop when `agg.n as f64 > graph.n as f64 * (1.0 - min_compression_ratio)`.
- Added hard depth cap: return when `depth >= max_recursion_depth` with a WARN log.
- Added DEBUG log when the compression-ratio cutoff fires.

### New Config Knobs
- `BoundariesConfig::leiden_min_compression_ratio` (default 0.1, range [0.0, 1.0)).
- `BoundariesConfig::leiden_max_recursion_depth` (default 32, min 1).
- Both wired through `LeidenConfig`, `LeidenConfigInput`, and `WasmLeidenConfigInput`.
- Config validation in `load_with_paths`.

### Code Organisation
- Extracted `local_move_phase`, `best_neighbour_community`, `compute_gain` from
  `leiden/mod.rs` to a new `leiden/local_move.rs` to keep mod.rs under 300 lines.
- Moved `ThresholdOverrideInput` / `ThresholdsInput` from `sdivi-core/input/types.rs`
  to `sdivi-core/input/threshold_input.rs`.
- Moved `WasmThresholdOverrideInput` / `WasmThresholdsInput` / `WasmThresholdBreachInfo`
  / `WasmAppliedOverrideInfo` / `WasmThresholdCheckResult` to
  `bindings/sdivi-wasm/src/threshold_types.rs`.

### New Tests
- `crates/sdivi-detection/tests/leiden_perf.rs` — sparse 1500-node graph, <2s wall-clock.
- `crates/sdivi-detection/tests/induced_subgraph.rs` — full unit suite for induced_subgraph.

### Documentation
- CHANGELOG.md entry.
- MIGRATION_NOTES.md entry.
- .tekhton/DRIFT_LOG.md entry.

## Root Cause (bugs only)
- Bug 1: `refine_community` in `refine.rs` allocated state for the entire graph
  (`vec![false; n]`, `RefinementState::from_partition` iterating all n nodes + all edges),
  even though refinement only touches nodes in the coarse community. On sparse graphs
  with hundreds of small coarse communities, this ran hundreds of times per outer-loop
  iteration and dominated runtime by 2-3 orders of magnitude.
- Bug 2: `leiden_recursive` in `mod.rs` only broke recursion when `agg_graph.n >= graph.n`
  (zero nodes merged). On sparse graphs that merge only a few nodes per level,
  this never fired, causing indefinite recursion even when no structural progress
  was being made.

## Files Modified
- `crates/sdivi-detection/src/leiden/graph.rs` — add `induced_subgraph`
- `crates/sdivi-detection/src/leiden/refine.rs` — rewrite `refine_community`
- `crates/sdivi-detection/src/leiden/local_move.rs` (NEW) — extracted from mod.rs
- `crates/sdivi-detection/src/leiden/mod.rs` — updated `leiden_recursive`
- `crates/sdivi-detection/src/partition.rs` — new fields on `LeidenConfig`
- `crates/sdivi-config/src/config.rs` — new fields on `BoundariesConfig`
- `crates/sdivi-config/src/load.rs` — validation for new fields
- `crates/sdivi-core/src/input/threshold_input.rs` (NEW) — extracted from types.rs
- `crates/sdivi-core/src/input/types.rs` — new fields on `LeidenConfigInput`
- `crates/sdivi-core/src/input/mod.rs` — updated imports
- `crates/sdivi-core/src/compute/boundaries.rs` — wire new fields through
- `bindings/sdivi-wasm/src/threshold_types.rs` (NEW) — extracted from types.rs
- `bindings/sdivi-wasm/src/types.rs` — new fields on `WasmLeidenConfigInput`
- `bindings/sdivi-wasm/src/lib.rs` — updated imports
- `crates/sdivi-core/tests/leiden_config_serde.rs` — updated struct literals
- `bindings/sdivi-wasm/tests/wasm_smoke.rs` — updated struct literals
- `bindings/sdivi-wasm/tests/wasm_weighted_leiden.rs` — updated struct literals
- `crates/sdivi-detection/tests/leiden_perf.rs` (NEW)
- `crates/sdivi-detection/tests/induced_subgraph.rs` (NEW)
- `CHANGELOG.md`
- `MIGRATION_NOTES.md`
- `.tekhton/DRIFT_LOG.md`

## Human Notes Status
N/A — no explicit Human Notes section in milestone.
