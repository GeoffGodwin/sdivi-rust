# Coder Summary
## Status: COMPLETE

## What Was Implemented

### M40: `collection_pipelines` pattern category

**`crates/sdivi-patterns/src/queries/collection_pipelines.rs`** (NEW)
- `NODE_KINDS: &[&str] = &[]` (call-based, no node-kind matching).
- `LazyLock<Regex>` member-call pattern: `\.(map|filter|reduce|flatMap|forEach|find|findIndex|some|every|flat)\(`.
- `matches_callee(text, language)` — TypeScript, JavaScript, Go, and Java; Python and Rust return `false`.
- Module doc covers: accepted receiver-type noise (RxJS Observable, ES6 Map/Set, DOM NodeList), pipe/compose seeds forward, disjointness from data_access and async_patterns.
- Inline unit tests: each method-name positive, chained pipeline positive, data_access negatives, async_patterns negatives, Math.max negative, bare-call negative, other-language negatives, `node_kinds_is_empty`.

**`crates/sdivi-patterns/src/queries/mod.rs`**
- Added `pub mod collection_pipelines;` (alphabetical, between `class_hierarchy` and `data_access`).
- Added `"collection_pipelines"` to `ALL_CATEGORIES` (13 → 14 entries).
- Added `collection_pipelines` to `CALL_DISPATCH` at slot P10 (after `data_access`).
- Updated dispatch order comment: P1/P4/P5/P6/P8/P9/P10 active at M40.
- Updated doc example: 13 → 14, added `collection_pipelines` assertion.

**`crates/sdivi-core/src/categories.rs`**
- Added `collection_pipelines` entry to `CATALOG_ENTRIES` (between `class_hierarchy` and `data_access`).
- Added `CATALOG_ENTRIES[13].0` to `CATEGORIES` (13 → 14 entries).
- Updated CATEGORIES doc example: 13 → 14, added `collection_pipelines` assertion.
- Updated `list_categories` doc example to include `collection_pipelines`.

**`crates/sdivi-patterns/src/queries/tests.rs`**
- Renamed `all_categories_has_thirteen_entries` → `all_categories_has_fourteen_entries` (13 → 14).
- Added `collection_pipelines` to assertion.
- Added M40 section: `xs_map_is_collection_pipelines`, `db_query_is_data_access_not_collection_pipelines`, `promise_then_is_async_not_collection_pipelines`.

**`crates/sdivi-core/tests/category_contract.rs`**
- Renamed count test: 13 → 14 (`list_categories_returns_exactly_fourteen_categories`).

**`crates/sdivi-core/tests/category_contract_m40.rs`** (NEW)
- M40 acceptance criterion tests: `xs.map(f)` → `["collection_pipelines"]`, `db.query(sql)` → `["data_access"]`, all 9 method names individually, disjointness with data_access/async_patterns, wrong-language negatives, `list_categories_includes_collection_pipelines`.

**`crates/sdivi-patterns/tests/dispatch_disjointness.rs`**
- Added `collection_pipelines` to imports.
- Added `collection_pipelines::matches_callee` to `all_matching_categories` (updated comment M39 → M40).
- Added 11 CORPUS entries: 10 positive method-name examples + `client.read(buf)` negative.
- Added `collection_pipelines` to `known_overlaps_winner_matches_dispatch_order` loser match arm.

**`bindings/sdivi-wasm/tests/wasm_smoke.rs`**
- Updated count 13 → 14, added `collection_pipelines` name assertion.

**`docs/pattern-categories.md`**
- Added `collection_pipelines` row to canonical category list table.
- Added `collection_pipelines` row to TypeScript/JavaScript per-language table.
- Added `collection_pipelines::matches_callee(text, language)` callee-text section (before `framework_hooks`).
- Updated dispatch table P10 entry with full method list.
- Updated active slots comment: P1/P4/P5/P6/P8/P9/P10 active at M40.
- Updated documented overlaps comment: M39 → M40.
- Updated future overlaps: M40–M44 → M41–M44.
- Added embedder responsibility #12 for M40; renumbered old #12 → #13.

**`MIGRATION_NOTES.md`**
- Added M40 section (before M39): count-introduction event, receiver-type noise note, disjointness claim, escape hatch example.

**`CHANGELOG.md`**
- Added M40 entry under `[Unreleased] ### Added` (before M39 entry).

## Root Cause (bugs only)
N/A — new feature milestone.

## Files Modified
- `crates/sdivi-patterns/src/queries/collection_pipelines.rs` (NEW) — 201 lines
- `crates/sdivi-patterns/src/queries/mod.rs` — module, CALL_DISPATCH P10, ALL_CATEGORIES 14; 189 lines
- `crates/sdivi-patterns/src/queries/tests.rs` — count 14, M40 tests; 267 lines
- `crates/sdivi-core/src/categories.rs` — collection_pipelines CATALOG_ENTRIES, CATEGORIES[13]; 251 lines
- `crates/sdivi-core/tests/category_contract.rs` — count 14; 279 lines
- `crates/sdivi-core/tests/category_contract_m40.rs` (NEW) — M40 acceptance criterion tests; 166 lines
- `crates/sdivi-patterns/tests/dispatch_disjointness.rs` — collection_pipelines import, all_matching_categories, corpus entries, loser match arm; 278 lines
- `bindings/sdivi-wasm/tests/wasm_smoke.rs` — count 14, collection_pipelines name; 260 lines
- `docs/pattern-categories.md` — canonical list, TS/JS table, callee-text section, dispatch table, active slots, KNOWN_OVERLAPS update, embedder responsibility #12
- `MIGRATION_NOTES.md` — M40 section added
- `CHANGELOG.md` — M40 entry added

## Human Notes Status
- Non-blocking (M39 reviewer): `framework_hooks` CATALOG_ENTRIES description lists `useStore` as an example — NOT_ADDRESSED (out of scope for M40)
- Non-blocking (M39 reviewer): CODER_SUMMARY.md module placement note inaccuracy — NOT_ADDRESSED (prior milestone artifact)
- Coverage gap (M39 reviewer): wasm_smoke.rs missing `resource_management`, `state_management`, `type_assertions` — NOT_ADDRESSED (pre-existing, out of scope for M40)

## Docs Updated
- `docs/pattern-categories.md` — collection_pipelines in canonical list, TS/JS per-language table, callee-text section, dispatch order M40, embedder responsibility #12.
- `MIGRATION_NOTES.md` — M40 section (count-introduction, receiver-type noise, disjointness, escape hatch).
- `CHANGELOG.md` — M40 entry under `[Unreleased]`.

## Observed Issues (out of scope)
- Pre-existing: `wasm_package_json_version_matches_workspace` test failure — `package.json` stranded at older version (0.2.23 vs 0.2.31). Not introduced by M40.
- Pre-existing: Test name `null_safety_node_kinds_do_not_match_non_ts_js_languages` is semantically inverted (body asserts a match). Carry-over from M37.
- Pre-existing (M39 reviewer): `crates/sdivi-patterns/src/queries/mod.rs:84-106` doc comment for `category_for_node_kind` lists only `logging` as callee-only category; `framework_hooks`, `schema_validation`, and `state_store` (and now `collection_pipelines`) are also callee-only.
- Pre-existing (M39 reviewer): No blank line between closing `];` of `CALL_DISPATCH` and the `/// Classify…` doc block for `classify_hint`.
