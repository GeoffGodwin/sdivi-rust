# Coder Summary
## Status: COMPLETE

## What Was Implemented

Fixed the following genuinely open issues from the 57 non-blocking notes in NON_BLOCKING_LOG.md:

1. **`ALL_CATEGORIES` doc in `mod.rs`** — Rewrote the classification-paths note to correctly classify `data_access` and `concurrency` as "hybrid" (both node-kind and callee-text paths), not "callee-text only". The old doc incorrectly claimed `category_for_node_kind` never returns them, contradicting the actual dispatch code and the `call_expression_is_data_access` test.

2. **`check_docs.sh` comment** — Updated the "Files scanned" comment to say `examples/*.ts (glob — all current and future .ts examples)` instead of listing the two hardcoded filenames that were removed when the glob was added in a prior milestone.

3. **`fn?.()` misleading examples** (5 files) — Removed `fn?.()` from being presented as an `optional_chain` example, since optional calls actually emit `call_expression` in the tree-sitter grammar, not `optional_chain`. Added clarifying notes in each location:
   - `null_safety.rs` — updated const doc to note `fn?.()` emits `call_expression`
   - `categories.rs` — removed `fn?.()`, added brief note
   - `docs/pattern-categories.md` — updated null_safety row with clarifying note
   - `CHANGELOG.md` — removed from positively-matched list, added note
   - `MIGRATION_NOTES.md` — removed from example list, added note

4. **`select_statement` SQL adapter risk** in `concurrency.rs` — Added a documentation comment on `NODE_KINDS` noting that SQL grammars also emit `select_statement`, and that a future SQL adapter must not include it in `PATTERN_KINDS` to avoid misclassification.

5. **WASM `pkg-template/package.json` version** — Bumped from `0.2.40` to `0.2.41` to match the workspace version.

6. **Unresolved rustdoc links** in `bindings/sdivi-wasm/src/types.rs` — Replaced `[`crate::infer_boundaries`]` and `[`crate::compute_trend`]` intra-doc links with plain code-formatted text. These functions live in the private `exports` module, so the intra-doc links could not resolve and caused `cargo doc --workspace -D warnings` failures.

## Root Cause (bugs only)
N/A — this is a doc/test cleanup task addressing accumulated non-blocking reviewer notes.

## Files Modified
- `crates/sdivi-patterns/src/queries/mod.rs` — corrected ALL_CATEGORIES doc classification paths
- `bindings/sdivi-wasm/tests/check_docs.sh` — updated "Files scanned" comment to reflect glob
- `crates/sdivi-patterns/src/queries/null_safety.rs` — clarified fn?.() emits call_expression
- `crates/sdivi-core/src/categories.rs` — removed fn?.() from null_safety description
- `docs/pattern-categories.md` — updated null_safety canonical table row
- `CHANGELOG.md` — corrected null_safety entry to remove fn?.() as positive example
- `MIGRATION_NOTES.md` — corrected null_safety migration entry
- `crates/sdivi-patterns/src/queries/concurrency.rs` — added select_statement SQL adapter seed comment
- `bindings/sdivi-wasm/pkg-template/package.json` — synced version to 0.2.41
- `bindings/sdivi-wasm/src/types.rs` — fixed 3 unresolved rustdoc links (crate::infer_boundaries ×2, crate::compute_trend ×1)

## Human Notes Status

All 57 notes addressed or confirmed pre-fixed. Status per unique issue class:

### Items FIXED in this run:
- COMPLETED: `ALL_CATEGORIES` doc incorrectly lists `data_access` and `concurrency` as callee-only (notes 1, 6, 7, 11, 18, 19, 20, 24, 29, 33) — fixed in `mod.rs`
- COMPLETED: `check_docs.sh` comment lists specific filenames after glob change (notes 2, 3) — fixed comment
- COMPLETED: `fn?.()` presented as `optional_chain` example in 5 locations (notes 30, 39) — fixed all 5
- COMPLETED: `select_statement` language guard note for SQL adapters (note 14/21) — added seed comment to `concurrency.rs`
- COMPLETED: WASM `pkg-template/package.json` version stranded (notes 12, 19, 36, 41, 46, 51) — bumped to 0.2.41
- COMPLETED: Unresolved rustdoc links in `types.rs` (note 53) — fixed 3 broken links

### Items confirmed ALREADY FIXED in prior milestones:
- ALREADY_FIXED: `comprehensions.rs` test rename (note 5) — already `non_comprehension_node_kinds_do_not_match`
- ALREADY_FIXED: `m23_native.rs` stale function name (notes 13, 15, 17, 22, 30) — already `list_categories_returns_all_categories`
- ALREADY_FIXED: `wasm_smoke.rs` only checks 9 of 12 names (notes 27, 36) — now checks all 19
- ALREADY_FIXED: blank line before `classify_hint` doc (notes 13, 18, 19, 23, 28, 32, 38) — blank line already present
- ALREADY_FIXED: P10 label incorrectly on `decorators` (note 42) — already fixed in docs
- ALREADY_FIXED: KNOWN_OVERLAPS header reads "M34" (note 39) — already shows M44
- ALREADY_FIXED: stale assertion message "logging is catalog-only" (notes 35, 40, 43, 44, 45, 50, 52, 53) — already fixed
- ALREADY_FIXED: `classify_hint` doc says "M35" (note 41/55) — already says M44
- ALREADY_FIXED: `framework_hooks` uses `useStore` example (note 25) — already uses `useAuth`, `useTheme`
- ALREADY_FIXED: `sdivi-core/src/lib.rs` re-export docs missing `# Examples` (note 48) — already has `# Examples`
- ALREADY_FIXED: CHANGELOG `Vec<String>` vs `Vec<&'static str>` (note 47) — already says `Vec<&'static str>`
- ALREADY_FIXED: `sdivi-lang-rust/src/extract.rs` inline truncation (note 50) — already uses shared helper
- ALREADY_FIXED: `category_contract_m42.rs` stale comment (note 23) — already `list_categories_count_after_m42` with updated comment
- ALREADY_FIXED: `null_safety_node_kinds_do_not_match_non_ts_js_languages` name (notes 28, 31, 37, 40) — already renamed
- ALREADY_FIXED: `dispatch_disjointness.rs` misleading comment (note 33) — already has better comment explaining the `@` prefix
- ALREADY_FIXED: double-counting guard comment in TS/JS adapters (note 34) — guard comment already in both adapters

### Items NOT_ADDRESSED (informational only, no code change warranted):
- NOT_ADDRESSED: `wasm.yml` `npm install --no-save` at workspace root (note 4) — step is well-commented explaining intent; it installs `tsc` to workspace root's `node_modules` for subsequent `npx tsc`. Changing working directory would break the install path. Acceptable as-is.
- NOT_ADDRESSED: `tests_m45_2.rs` cross-tier redundancy (note 8) — acceptable practice per reviewer note itself
- NOT_ADDRESSED: `try_statement` fix within scope (note 9) — informational, no action
- NOT_ADDRESSED: Cargo.toml auto-discovery (note 10) — informational, tests already pass
- NOT_ADDRESSED: `CODER_SUMMARY.md` wasm_smoke coverage gap (note 22) — prior-session artifact, code is correct
- NOT_ADDRESSED: `CODER_SUMMARY.md` state_store alphabetical note (note 26) — prior-session artifact, not a code defect
- NOT_ADDRESSED: `docs/pattern-categories.md` Go/Java table incomplete (note 51) — pre-existing gap, out of scope
- NOT_ADDRESSED: `MIGRATION_NOTES.md` worked example confirmation (note 46) — milestone-era validation note
- NOT_ADDRESSED: `CHANGELOG.md` prior count stale note (note 49) — historical count was accurate when written
- NOT_ADDRESSED: `prop_classify_hint.rs` absent from prior CODER_SUMMARY (note 45) — prior-session accuracy gap only

## Observed Issues (out of scope)
None observed.
