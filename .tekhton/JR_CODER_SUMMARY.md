# Jr Coder Summary

**Date:** 2026-05-02  
**Task:** Architect Remediation — Staleness Fixes

## Changes Made

### 1. crates/sdivi-core/src/compute/mod.rs:9
**Change:** Added inline comment to `mod threshold_types` line.

**Before:**
```rust
mod threshold_types;
```

**After:**
```rust
// Private module: types are re-exported publicly via `pub use super::threshold_types::*` in `thresholds.rs`.
mod threshold_types;
```

**Rationale:** Explains why `threshold_types` is intentionally private while surrounded by `pub mod` declarations. Resolves duplicate M19 and M20 drift-log observations about perceived asymmetry.

---

### 2. crates/sdivi-pipeline/src/helpers.rs:57
**Change:** Added comment on the `(0..n)` range in `graph_to_boundary_input` documenting the contiguous-index invariant.

**Before:**
```rust
let n = dg.node_count();
let ids: Vec<String> = (0..n)
```

**After:**
```rust
let n = dg.node_count();
// This loop assumes DependencyGraph maintains a contiguous 0..node_count() node-index space.
// If node removal is ever added to DependencyGraph, this assumption must be revisited to avoid silent node drops.
let ids: Vec<String> = (0..n)
```

**Rationale:** Documents the hidden invariant that prevents silent node drops if `DependencyGraph` ever adds node-removal capability. Resolves M19 drift-log observation about implicit assumptions in `helpers.rs:55-70`.

---

## Drift Log Resolutions

These fixes address the following drift-log entries (mark as **RESOLVED** in DRIFT_LOG.md):

- `compute/mod.rs:9` (M20 run) — threshold_types private module asymmetry
- `compute/mod.rs:9` (M19 run) — duplicate observation
- `helpers.rs:55-70` (M19 run) — contiguous-index assumption

All fixes are mechanical documentation additions with no code logic changes.
