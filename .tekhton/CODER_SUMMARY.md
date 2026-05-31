# Coder Summary
## Status: COMPLETE

## What Was Implemented

### M35: `framework_hooks` pattern category

**`crates/sdivi-patterns/src/queries/framework_hooks.rs`** (NEW)
- `NODE_KINDS: &[&str] = &[]` (callee-only; no node-kind matching)
- `static TS_JS_RE: LazyLock<Regex>` for `^use[A-Z]`
- `pub fn matches_callee(text, language) -> bool` — TypeScript/JavaScript only
- Inline unit tests for built-in hooks, custom hooks, negative cases, wrong languages

**`crates/sdivi-patterns/src/queries/mod.rs`**
- Added `pub mod framework_hooks;`
- Added `"framework_hooks"` to `ALL_CATEGORIES` (alphabetical position: after `error_handling`)
- Added `("framework_hooks", framework_hooks::matches_callee)` to `CALL_DISPATCH` at slot P6 (between async_patterns P1 and logging P8)
- Updated dispatch-order doc: `(P1/P6/P8/P9 active at M35)`
- Updated `CALL_DISPATCH` inline comment to list P6
- Updated `ALL_CATEGORIES` doc example: count 8 → 9, swapped assertions for `framework_hooks`
- Renamed test `all_categories_has_eight_entries` → `all_categories_has_nine_entries`, updated count
- Removed stale cross-reference comment to keep file at exactly 300 lines

**`crates/sdivi-core/src/categories.rs`**
- Added `framework_hooks` entry to `CATALOG_ENTRIES` (position 4, alphabetical)
- Updated `CATEGORIES` const to index `CATALOG_ENTRIES[0..8].0` (9 entries)
- Updated doc examples: count 8 → 9, `framework_hooks` assertion added

**`crates/sdivi-patterns/tests/framework_hooks.rs`** (NEW)
- `classify_hint` acceptance criteria: `useState(0)` → `["framework_hooks"]`, etc.
- `matches_callee` positive: all built-in hooks + custom hooks
- `matches_callee` negative: lowercase second char, non-`use` prefix, wrong languages

**`crates/sdivi-patterns/tests/dispatch_disjointness.rs`**
- Added `framework_hooks` to the `use` import
- Updated `all_matching_categories` to include `framework_hooks::matches_callee` (TODO(M35) fulfilled)
- Added 7 CORPUS entries: 5 positive (TypeScript/JavaScript hooks), 2 negative
- Added `"framework_hooks" => framework_hooks::matches_callee(text, lang)` arm to `loser_matches`
- Updated TODO comment to reference M39 (next slot)

**`docs/pattern-categories.md`**
- Added `framework_hooks` row to canonical category list table
- Added `framework_hooks` row to TypeScript/JavaScript node-kind mapping table
- Added `framework_hooks::matches_callee` regex section with worked example
- Updated dispatch order note: P6 now active at M35
- Updated future-overlaps section (M35 overlap with M39 state_store documented)
- Added embedder-responsibility item #7 for `framework_hooks`

**`MIGRATION_NOTES.md`**
- Added M35 section: count-introduction event, escape hatch, trend continuity

**`CHANGELOG.md`**
- Added `### Added` entry for `framework_hooks` under `[Unreleased]`

## Root Cause (bugs only)
N/A — feature addition

## Files Modified
- `crates/sdivi-patterns/src/queries/framework_hooks.rs` (NEW) — 119 lines
- `crates/sdivi-patterns/src/queries/mod.rs` — 300 lines (unchanged ceiling)
- `crates/sdivi-core/src/categories.rs` — 180 lines
- `crates/sdivi-patterns/tests/framework_hooks.rs` (NEW) — 167 lines
- `crates/sdivi-patterns/tests/dispatch_disjointness.rs` — 203 lines
- `docs/pattern-categories.md`
- `MIGRATION_NOTES.md`
- `CHANGELOG.md`

## Human Notes Status
No human notes in this task.

## Docs Updated
- `docs/pattern-categories.md` — canonical category list, TS/JS node-kind table, `framework_hooks::matches_callee` regex section, dispatch order updated to P6 active at M35, future-overlaps and embedder-responsibilities updated.
- `MIGRATION_NOTES.md` — M35 section added (count-introduction event).
- `CHANGELOG.md` — Added entry under `[Unreleased]`.

## Observed Issues (out of scope)
- `wasm_package_json_version_matches_workspace` test failure — pre-existing before this run (wasm `package.json` stranded at 0.2.23, workspace at 0.2.24). Not introduced by M35.
- `crates/sdivi-patterns/src/queries/mod.rs` — stale phrasing in line 279 assertion message ("logging is catalog-only in v0 for category_for_node_kind"). Pre-existing; previously flagged by M23 and M34 reviewers.
