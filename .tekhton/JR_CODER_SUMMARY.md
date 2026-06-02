# Jr Coder Summary — Architect Remediation

**Date:** 2026-06-02  
**Branch:** feature/MorePatterns  
**Status:** Complete

## What Was Fixed

### Staleness Fixes

1. **`docs/pattern-categories.md` — embedder responsibilities list numbering regression** (items 14–17)
   - Reordered four misplaced items to sequential numbering
   - M42 testing (was item 14, now at line 403) → moved to position 14
   - M43 serialization (was item 15) → remains item 15
   - M44 concurrency (was item 16) → remains item 16
   - class_hierarchy note (was item 15) → renumbered to item 17

2. **`docs/pattern-categories.md` — canonical category table alphabetical order** (rows 3–5 after class_hierarchy)
   - Reordered three rows in the canonical category list table
   - Before: concurrency, collection_pipelines, comprehensions
   - After: collection_pipelines, comprehensions, concurrency (alphabetical)

3. **`bindings/sdivi-wasm/pkg-template/package.json` — version sync**
   - Updated `"version": "0.2.43"` → `"version": "0.2.44"`
   - Now matches workspace `[workspace.package].version`
   - Test `wasm_package_json_version_matches_workspace` now passes

4. **`crates/sdivi-patterns/src/queries/mod.rs:137` — split priority-slot comment**
   - Moved 11-slot dispatch order comment to a separate line above the `#[allow]` attribute
   - New format:
     ```rust
     // Dispatch order: P1 > P2=testing > P3=serialization > … > P11=concurrency; future milestones insert at their slot
     #[allow(clippy::type_complexity)] // fn pointer tuple slice; each entry is one category
     const CALL_DISPATCH: &[(&str, fn(&str, &str) -> bool)] = &[
     ```

5. **`crates/sdivi-patterns/src/queries/mod.rs:94** — `category_for_node_kind` doc asymmetry note**
   - Added two sentences explaining `call_expression` asymmetry
   - Notes that this node kind always returns `"data_access"` here because callee text is unavailable
   - Recommends using `classify_hint` for callee-aware results

### Dead Code Removal

6. **`crates/sdivi-pipeline/tests/snapshot_m32_unchanged.rs` → `snapshot_pipeline_regression.rs`**
   - Renamed test file to reflect actual contents (M32 + M33 tests)
   - All test functions and doc comments already encoded milestone context
   - No CI or doc references to old filename found (only in `.claude/logs/`)
   - All 4 tests pass with new name

## Files Modified

1. `/home/geoff/workspace/geoffgodwin/sdivi-rust/docs/pattern-categories.md`
   - Fixed embedder list numbering (14–17)
   - Reordered canonical category table (collection_pipelines, comprehensions, concurrency)

2. `/home/geoff/workspace/geoffgodwin/sdivi-rust/bindings/sdivi-wasm/pkg-template/package.json`
   - Version bump: 0.2.43 → 0.2.44

3. `/home/geoff/workspace/geoffgodwin/sdivi-rust/crates/sdivi-patterns/src/queries/mod.rs`
   - Dispatch order comment split across two lines
   - `category_for_node_kind` doc enhanced with asymmetry note

4. `/home/geoff/workspace/geoffgodwin/sdivi-rust/crates/sdivi-pipeline/tests/snapshot_m32_unchanged.rs` → `snapshot_pipeline_regression.rs`
   - File renamed (no content changes)

## Verification

- ✅ `cargo test --workspace --lib` — all library tests pass
- ✅ `cargo test --test snapshot_pipeline_regression` — renamed test runs, all 4 tests pass
- ✅ `cargo test wasm_package_json_version_matches_workspace` — version test passes
- ✅ `cargo test dispatch_disjointness` — dispatch order tests pass
- ✅ `cargo test test_all_categories_doc_classification` — category doc tests pass

## Notes

- No items from "Simplification" (route to sr coder) or "Design Doc Observations" (route to human) were touched
- All changes are mechanical and bounded per instructions
- No refactoring, judgment calls, or multi-file dependencies introduced
