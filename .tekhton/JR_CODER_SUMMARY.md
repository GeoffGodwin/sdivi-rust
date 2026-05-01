# Jr Coder Summary
**Date:** 2026-05-01
**Cycle:** post-M16 architect remediation

---

## Changes Applied

### Staleness Fixes (4 items)

1. **`crates/sdi-core/src/input/edge_weight.rs:9–14`** — Updated `edge_weight_key` doc comment
   - Removed false claim that "Leiden detection code will produce incorrect weights if [source < target invariant] is violated"
   - Replaced with accurate statement: "Callers should canonicalize; mis-ordered pairs are normalized by the detection layer, so no weight is silently discarded"
   - Reflects implementation in `boundaries.rs:109` which normalizes reversed pairs

2. **`bindings/sdi-wasm/src/types.rs:46`** — Strengthened `WasmLeidenConfigInput` doc comment
   - Changed from: "Leiden algorithm configuration. `edge_weights` is absent — WASM Leiden is always unweighted."
   - To: Multi-line comment explicitly labeling gap as INTENTIONAL and referencing ADL-4
   - Surfaces intentional divergence as searchable for future editors adding fields to `LeidenConfigInput`

3. **`crates/sdi-pipeline/src/change_coupling.rs:86–88`** — Clarified `parse_git_log_output` doc comment
   - Simplified from: unclear "NUL-separated output" with confusing example
   - To: Clear distinction between NUL-delimited commit-header tokens and newline-separated filenames
   - Aligns doc with actual code behavior (`.lines()` on line 111)

4. **`crates/sdi-detection/src/leiden/quality.rs:1`** — Added missing module-level doc
   - Added `//!` comment: "Per-community stability (internal edge density) computation."
   - Consistent with sibling modules `graph.rs`, `modularity.rs`, `move_nodes.rs`

---

### Dead Code Removal (3 items)

1. **`crates/sdi-core/src/compute/boundaries.rs:160,173–174`** — Removed unused stub code
   - Removed line 160: `let current_communities = invert_assignments(current);`
   - Removed lines 173–174: comment "Also measure how similar..." and `let _ = &current_communities;`
   - `invert_assignments` has no side effects; stub was placeholder for future work (belongs in issue tracker, not source)

2. **`crates/sdi-patterns/src/catalog.rs:133,146`** — Added feature gates to dead code in `--no-default-features` build
   - Added `#[cfg(feature = "pipeline-records")]` above `build_globset` (line 133)
   - Added `#[cfg(feature = "pipeline-records")]` above `is_excluded` (line 146)
   - Both helpers are only called from `build_catalog`, which is gated by the same feature
   - Prevents compiler warnings in WASM/`--no-default-features` builds

3. **`tests/historical_commit_lifecycle.rs`** — Deleted comment-only placeholder file
   - File was 7 lines: comment explaining the actual test is at `crates/sdi-cli/tests/historical_commit_lifecycle.rs`
   - Workspace-level `tests/` files cannot be compiled by Cargo without root `[package]`
   - Canonical test already exists and is self-documenting; file adds no value

---

## Verification

All changes are mechanical and bounded:
- No logic changes, only doc comment updates and feature gates
- All referenced implementation details verified in source (e.g., `boundaries.rs:109`, `catalog.rs:88`)
- Feature gate additions match the gating of their only caller

---

## Next Steps

The drift log entries should be updated to mark these as RESOLVED:
- Entries 1, 4, 5, 7, 8, 9, 10 are addressed by this PR
- Entries 2 and 3 (stale observations) should be marked resolved without code change
