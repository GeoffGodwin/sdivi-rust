# Coder Summary
## Status: COMPLETE

## What Was Implemented

### M39: `state_store` pattern category

**`crates/sdivi-patterns/src/queries/state_store.rs`** (NEW)
- `NODE_KINDS: &[&str] = &[]` (call-based, no node-kind matching).
- `LazyLock<Regex>` TS/JS table with `^`-anchored patterns: Redux/RTK factories, React-Redux hooks (`useSelector`/`useDispatch`/`useStore`), Zustand bare `create(`, Jotai/Recoil, MobX, Signals, Solid createX.
- `matches_callee(text, language)` — TypeScript/JavaScript only; other languages return `false`.
- Module doc covers: all library families, `^`-anchor rationale, accepted noise (local create/effect), open question (TanStack Query / SWR).
- Inline unit tests: Redux/RTK positives, React-Redux hook positives, Zustand/Jotai/MobX/Signals/Solid positives, member-access negatives, unrelated-hook negatives, other-language negatives, `node_kinds_is_empty`.

**`crates/sdivi-patterns/src/queries/mod.rs`**
- Added `pub mod state_store;` (alphabetical, between `schema_validation` and `state_management`).
- Added `"state_store"` to `ALL_CATEGORIES` (12 → 13 entries).
- Added `state_store` to `CALL_DISPATCH` at slot P5 (between schema_validation P4 and framework_hooks P6).
- Updated dispatch order comment: P1/P4/P5/P6/P8/P9 active at M39.
- Updated doc example: 12 → 13, added `state_store` assertion.

**`crates/sdivi-core/src/categories.rs`**
- Added `state_store` entry to `CATALOG_ENTRIES` (between `state_management` and `type_assertions`).
- Added `CATALOG_ENTRIES[12].0` to `CATEGORIES` (12 → 13 entries).
- Updated CATEGORIES doc example: 12 → 13, added `state_store` assertion.
- Updated `list_categories` doc example to include `state_store`.

**`crates/sdivi-patterns/src/queries/tests.rs`**
- Renamed `all_categories_has_twelve_entries` → `all_categories_has_thirteen_entries` (12 → 13).
- Added `state_store` to assertion.
- Added M39 section: `use_selector_is_state_store_not_framework_hooks`, `create_slice_is_state_store`, `use_effect_is_still_framework_hooks`.

**`crates/sdivi-core/tests/category_contract.rs`**
- Renamed count test: 12 → 13 (`list_categories_returns_exactly_thirteen_categories`).

**`crates/sdivi-core/tests/category_contract_m39.rs`** (NEW)
- M39 acceptance criterion tests: `useSelector` → `["state_store"]` (NOT `framework_hooks`), `createSlice` → `["state_store"]`, `useEffect` → `["framework_hooks"]`, member-access negatives, wrong-language negatives.

**`crates/sdivi-patterns/tests/dispatch_disjointness.rs`**
- Added `state_store` to imports.
- Added `state_store::matches_callee` to `all_matching_categories` (updated comment M38 → M39).
- Added 12 CORPUS entries: state_store positives + hook/store disambiguation + member-access negative.
- Added 3 KNOWN_OVERLAPS entries: `useSelector`/`useDispatch`/`useStore` — state_store (P5) beats framework_hooks (P6).
- Updated `known_overlaps_winner_matches_dispatch_order` loser match to include `state_store`.

**`bindings/sdivi-wasm/tests/wasm_smoke.rs`**
- Updated count 12 → 13, added `state_store` name assertion.

**`docs/pattern-categories.md`**
- Added `state_store` row to canonical category list table.
- Added `state_store` row to TypeScript/JavaScript per-language table.
- Added `state_store::matches_callee(text, language)` callee-text section (before `framework_hooks`).
- Updated dispatch table comment: P1/P4/P5/P6/P8/P9 active at M39.
- Updated KNOWN_OVERLAPS section with three `useSelector`/`useDispatch`/`useStore` overlap entries.
- Added embedder responsibility #11 for M39 (precedence reassignment story); renumbered old #11 → #12.

**`MIGRATION_NOTES.md`**
- Added M39 section (before M38): count-introduction event, precedence reassignment story, `^`-anchor rationale, TanStack Query open question, escape hatch, trend continuity.

**`CHANGELOG.md`**
- Added M39 entry under `[Unreleased] ### Added` (before M38 entry).

## Root Cause (bugs only)
N/A — new feature milestone.

## Files Modified
- `crates/sdivi-patterns/src/queries/state_store.rs` (NEW) — 234 lines
- `crates/sdivi-patterns/src/queries/mod.rs` — module, CALL_DISPATCH P5, ALL_CATEGORIES 13; 185 lines
- `crates/sdivi-patterns/src/queries/tests.rs` — count 13, M39 tests; 237 lines
- `crates/sdivi-core/src/categories.rs` — state_store CATALOG_ENTRIES, CATEGORIES[12]; 236 lines
- `crates/sdivi-core/tests/category_contract.rs` — count 13; 279 lines
- `crates/sdivi-core/tests/category_contract_m39.rs` (NEW) — M39 acceptance criterion tests; 182 lines
- `crates/sdivi-patterns/tests/dispatch_disjointness.rs` — state_store import, all_matching_categories, corpus entries, KNOWN_OVERLAPS; 261 lines
- `bindings/sdivi-wasm/tests/wasm_smoke.rs` — count 13, state_store name; 259 lines
- `docs/pattern-categories.md` — canonical list, TS/JS table, callee-text section, dispatch table, KNOWN_OVERLAPS, embedder responsibility
- `MIGRATION_NOTES.md` — M39 section added
- `CHANGELOG.md` — M39 entry added

## Human Notes Status
No human notes in this run.

## Docs Updated
- `docs/pattern-categories.md` — state_store in canonical list, TS/JS per-language table, callee-text section, dispatch order M39, KNOWN_OVERLAPS entries, embedder responsibility #11.
- `MIGRATION_NOTES.md` — M39 section (count-introduction, precedence reassignment, escape hatch).
- `CHANGELOG.md` — M39 entry under `[Unreleased]`.

## Observed Issues (out of scope)
- Pre-existing: `wasm_package_json_version_matches_workspace` test failure — `package.json` stranded at older version. Not introduced by M39.
- Pre-existing: Test name `null_safety_node_kinds_do_not_match_non_ts_js_languages` is semantically inverted (body asserts a match). Carry-over from M37.
- Pre-existing: Reviewer noted `fn?.()` example in M37 docs (`null_safety.rs`, `categories.rs`, `docs/pattern-categories.md`) but grammar emits `call_expression` for optional calls. Doc is accurate to spec but inconsistent with grammar reality.
