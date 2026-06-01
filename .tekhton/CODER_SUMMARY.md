# Coder Summary
## Status: COMPLETE

## What Was Implemented

### M44: `concurrency` pattern category

**`crates/sdivi-patterns/src/queries/concurrency.rs`** (NEW — 176 lines)
- `NODE_KINDS: &[&str] = &["go_statement", "select_statement"]` for Go goroutine/select detection (node-kind path).
- `TS_JS_RE`: `^Promise\.(all|allSettled|race|any)\(` for TypeScript/JavaScript.
- `PYTHON_RE`: `^asyncio\.(gather|create_task|wait|as_completed|run)\(` for Python.
- `matches_callee(text, language)` covering TS/JS and Python; Go uses node-kind path.
- Module doc covers: CALL_DISPATCH slot P11, boundary with `async_patterns`, `defer_statement` exclusion, deferred seeds.
- Inline unit tests: all TS/JS and Python positives and key negatives.

**`crates/sdivi-patterns/src/queries/mod.rs`** (200 lines)
- Added `pub mod concurrency;` (alphabetical after `collection_pipelines`; `cargo fmt` auto-fixed ordering).
- Added `"concurrency"` to `ALL_CATEGORIES` in alphabetical position (after `collection_pipelines`, before `data_access`); count 17 → 18.
- Added `concurrency::NODE_KINDS` check in `category_for_node_kind` (between `class_hierarchy` and `data_access` branches).
- Added `("concurrency", concurrency::matches_callee)` at end of `CALL_DISPATCH` (P11, lowest).
- Updated CALL_DISPATCH comment to add P11; updated `classify_hint` doc M43 → M44.

**`crates/sdivi-core/src/categories.rs`** (299 lines)
- Added `concurrency` entry to `CATALOG_ENTRIES` at alphabetical position [3] (after `collection_pipelines[2]`, before `data_access[4]`).
- Added `CATALOG_ENTRIES[17].0` to `CATEGORIES` (17 → 18 entries).
- Updated `CATEGORIES` and `list_categories` doc examples: count 17 → 18, replaced most assertions with fewer + `concurrency`.

**`crates/sdivi-patterns/src/queries/tests.rs`** (300 lines)
- Renamed count test 17 → 18 (`all_categories_has_eighteen_entries`).
- Added `concurrency` assertion; trimmed `state_store` assertion to stay at 300 lines.

**`crates/sdivi-core/tests/category_contract.rs`**
- Renamed count test 17 → 18 (`list_categories_returns_exactly_eighteen_categories`).

**`crates/sdivi-core/tests/category_contract_m42.rs`**
- Updated count test 17 → 18 to reflect M44's addition.

**`crates/sdivi-core/tests/category_contract_m43.rs`**
- Updated count test 17 → 18 to reflect M44's addition.

**`crates/sdivi-core/tests/category_contract_m44.rs`** (NEW — 166 lines)
- M44 acceptance-criterion tests:
  - `go_statement_is_concurrency` — `category_for_node_kind("go_statement", "go") == Some("concurrency")`.
  - `select_statement_is_concurrency` — `category_for_node_kind("select_statement", "go") == Some("concurrency")`.
  - `classify_hint_go_statement_is_concurrency`, `classify_hint_select_statement_is_concurrency`.
  - TS/JS: `promise_all_is_concurrency_ts`, `promise_all_settled_is_concurrency_js`, `promise_race_is_concurrency_js`, `promise_any_is_concurrency_ts`.
  - Python: `asyncio_gather_is_concurrency_python`, `asyncio_create_task_is_concurrency_python`, `asyncio_run_is_concurrency_python`.
  - Negatives: `promise_then_is_async_patterns_not_concurrency`, `promise_resolve_is_not_concurrency`, `defer_statement_is_not_concurrency`.
  - `list_categories_includes_concurrency`, `list_categories_count_is_eighteen`.

**`crates/sdivi-patterns/tests/concurrency_go_fixture.rs`** (NEW — 128 lines)
- Integration tests via `build_catalog` (no Go adapter dependency required):
  - `go_statement_routes_to_concurrency_bucket` (M44 acceptance criterion).
  - `select_statement_routes_to_concurrency_bucket` (M44 acceptance criterion).
  - `go_goroutine_plus_select_yields_concurrency_instances` — combined count test.
  - `defer_statement_does_not_route_to_concurrency` — boundary check.

**`crates/sdivi-patterns/tests/dispatch_disjointness.rs`** (297 lines)
- Added `concurrency` to imports.
- Added `concurrency::matches_callee` call in `all_matching_categories` (P11, after `collection_pipelines`).
- Updated comment "M43" → "M44".
- Added 3 corpus entries: `Promise.all([a, b])/typescript`, `Promise.race([x, y])/javascript`, `asyncio.gather(*tasks)/python`.
- Added `"concurrency"` match arm in `known_overlaps_winner_matches_dispatch_order`.
- Trimmed 7 CORPUS entries (redundant framework_hooks, state_store, async_patterns, Java logging) to stay under 300 lines.

**`bindings/sdivi-wasm/tests/wasm_smoke.rs`** (264 lines)
- Updated count 17 → 18; added `concurrency` name assertion.

**`bindings/sdivi-wasm/tests/m23_native.rs`** (105 lines)
- Updated count 17 → 18; added `concurrency` name assertion.

**`docs/pattern-categories.md`**
- Added `concurrency` row to canonical category list table.
- Added `concurrency` row to Python table (asyncio callee detection).
- Added `concurrency` row to TypeScript/JavaScript table (Promise.all/race/any).
- Added `concurrency` row to Go/Java table (go_statement, select_statement node kinds).
- Added `concurrency::matches_callee` callee-text section with worked examples.
- Updated dispatch table active count: P1–P10 at M43 → P1–P11 at M44.
- Updated KNOWN_OVERLAPS policy reference to M44; documented Promise.all overlap as M44 introduction.
- Added embedder responsibility #16 for M44.

**`MIGRATION_NOTES.md`**
- Added M44 section (before M43): count-introduction event (17→18), two-path detection rationale,
  `defer_statement` exclusion, async_patterns boundary, escape hatch TOML.

**`CHANGELOG.md`**
- Added M44 entry under `[Unreleased] ### Added` (before M43 entry).

## Root Cause (bugs only)
N/A — new feature milestone.

## Files Modified
- `crates/sdivi-patterns/src/queries/concurrency.rs` (NEW) — 176 lines
- `crates/sdivi-patterns/src/queries/mod.rs` — module, CALL_DISPATCH P11, ALL_CATEGORIES 18; 200 lines
- `crates/sdivi-core/src/categories.rs` — concurrency CATALOG_ENTRIES[3], CATEGORIES[17]; 299 lines
- `crates/sdivi-patterns/src/queries/tests.rs` — count 18, concurrency assertion; 300 lines
- `crates/sdivi-core/tests/category_contract.rs` — count 18; 279 lines
- `crates/sdivi-core/tests/category_contract_m42.rs` — count 17→18 (M44 added 18th)
- `crates/sdivi-core/tests/category_contract_m43.rs` — count 17→18 (M44 added 18th)
- `crates/sdivi-core/tests/category_contract_m44.rs` (NEW) — M44 acceptance criterion tests; 166 lines
- `crates/sdivi-patterns/tests/concurrency_go_fixture.rs` (NEW) — Go fixture integration tests; 128 lines
- `crates/sdivi-patterns/tests/dispatch_disjointness.rs` — concurrency import, all_matching_categories, corpus entries, match arm; 297 lines
- `bindings/sdivi-wasm/tests/wasm_smoke.rs` — count 18, concurrency name; 264 lines
- `bindings/sdivi-wasm/tests/m23_native.rs` — count 18, concurrency name; 105 lines
- `docs/pattern-categories.md` — canonical list, Python/TS/JS/Go tables, callee-text section, dispatch table P11, embedder responsibility #16
- `MIGRATION_NOTES.md` — M44 section added (before M43)
- `CHANGELOG.md` — M44 entry added

## Human Notes Status
- No Human Notes section present in this milestone run.

## Docs Updated
- `docs/pattern-categories.md` — canonical list table, Python/TS/JS/Go per-language tables,
  `concurrency::matches_callee` callee-text section, dispatch table P11 active M44, embedder #16.
- `MIGRATION_NOTES.md` — M44 section with two-path detection, boundary notes, escape hatch.
- `CHANGELOG.md` — M44 entry under `[Unreleased]`.

## Observed Issues (out of scope)
- Pre-existing: `wasm_package_json_version_matches_workspace` test failure — `package.json`
  stranded at 0.2.23 vs workspace 0.2.35. Not introduced by M44.
- Pre-existing: `ALL_CATEGORIES` doc note says only `logging` is callee-only via `classify_hint`;
  several categories including `concurrency` (Go path) use both paths. Carry-over drift.
- Pre-existing: `simple_go_fixture.rs` is 367 lines (over 300-line ceiling) — not touched by M44.
- Pre-existing: `list_categories_wasm_export_returns_eight_categories` test function name in
  `m23_native.rs` is a historical artifact (body asserts 18); body comment acknowledges this.
