# Coder Summary
## Status: COMPLETE

## What Was Implemented

### M41: `http_routing` pattern category

**`crates/sdivi-patterns/src/queries/http_routing.rs`** (NEW)
- `NODE_KINDS: &[&str] = &[]` (call-based, no node-kind matching).
- Three `LazyLock<Regex>` statics: `TS_JS_RE` (receiver-allowlist anchored), `GO_RE` (uppercase HTTP verbs), `PYTHON_RE` (`.add_url_rule\(`).
- `matches_callee(text, language)` — TypeScript/JavaScript, Go, Python; all others return `false`.
- Module doc covers: receiver-allowlist precision, NestJS/FastAPI decorator distinction, CALL_DISPATCH slot P7, Python `add_url_rule`, seeds forward (GraphQL/gRPC/tRPC).
- Inline unit tests: all TS/JS methods, all Go methods, Python, client-call negatives, wrong-language negatives, `node_kinds_is_empty`.

**`crates/sdivi-patterns/src/queries/mod.rs`**
- Added `pub mod http_routing;` (alphabetical, between `framework_hooks` and `logging`).
- Added `"http_routing"` to `ALL_CATEGORIES` (14 → 15 entries).
- Inserted `("http_routing", http_routing::matches_callee)` at P7 in `CALL_DISPATCH` (between `framework_hooks` P6 and `logging` P8).
- Updated dispatch order comment: P7=http_routing added.
- Updated doc example: 14 → 15, added `http_routing` assertion.

**`crates/sdivi-core/src/categories.rs`**
- Added `http_routing` entry to `CATALOG_ENTRIES` (between `framework_hooks` and `logging`).
- Added `CATALOG_ENTRIES[14].0` to `CATEGORIES` (14 → 15 entries).
- Updated CATEGORIES doc example: 14 → 15, added `http_routing` assertion.
- Updated `list_categories` doc example to include `http_routing`.

**`crates/sdivi-patterns/src/queries/tests.rs`**
- Renamed `all_categories_has_fourteen_entries` → `all_categories_has_fifteen_entries` (14 → 15).
- Added `http_routing` to assertion.
- Added M41 section: `app_get_is_http_routing_not_data_access`, `axios_get_is_data_access_not_http_routing`, `go_http_handle_func_is_http_routing`.

**`crates/sdivi-core/tests/category_contract.rs`**
- Renamed count test: 14 → 15 (`list_categories_returns_exactly_fifteen_categories`).

**`crates/sdivi-core/tests/category_contract_m41.rs`** (NEW)
- M41 acceptance criterion tests: `app.get('/u', h)` → `["http_routing"]`, `axios.get(url)` → `["data_access"]`, all method/language positives and negatives, `list_categories_includes_http_routing`. 15 tests total.

**`crates/sdivi-patterns/tests/dispatch_disjointness.rs`**
- Added `http_routing` to imports.
- Added `http_routing::matches_callee` to `all_matching_categories` (updated comment M40 → M41).
- Added 2 KNOWN_OVERLAPS entries: `app.get('/u', h)` and `router.post('/user', cb)` both match http_routing (P7, winner) and data_access (P9, loser).
- Added 3 CORPUS entries: `app.get('/u', h)` → http_routing, `r.GET("/users", h)` → http_routing, `axios.get(url)` → data_access.
- Added `"http_routing"` to the match arm in `known_overlaps_winner_matches_dispatch_order`.

**`bindings/sdivi-wasm/tests/wasm_smoke.rs`**
- Updated count 14 → 15, added `http_routing` name assertion.

**`bindings/sdivi-wasm/tests/m23_native.rs`**
- Updated count assertion from 8 to 15 (pre-existing drift since this test was written at M23 and never updated through M34–M40; the workspace test was broken before M41 too — fixed as part of the M41 WASM-count-test requirement).
- Added `http_routing` name assertion.

**`docs/pattern-categories.md`**
- Added `http_routing` row to canonical category list table.
- Added `http_routing` row to TypeScript/JavaScript per-language table.
- Added `http_routing` row to Go/Java per-language table.
- Added `http_routing::matches_callee(text, language)` callee-text section (before `collection_pipelines`).
- Updated dispatch table P7 entry from future placeholder to active (`**M41**`).
- Updated active slots comment: M40 → M41.
- Added 2 KNOWN_OVERLAPS rows for `app.get` / `router.post`.
- Removed future-overlaps note about http_routing (now documented as active).
- Added embedder responsibility #13 for M41; renumbered old #13 → #14.

**`MIGRATION_NOTES.md`**
- Added M41 section (before M40): count-introduction event, headline before/after example (`app.get` → `http_routing`), client-call unaffected note, decorator-style routes note, idiosyncrasy gap, escape hatch example.

**`CHANGELOG.md`**
- Added M41 entry under `[Unreleased] ### Added` (before M40 entry).

## Root Cause (bugs only)
N/A — new feature milestone.

## Files Modified
- `crates/sdivi-patterns/src/queries/http_routing.rs` (NEW) — 267 lines
- `crates/sdivi-patterns/src/queries/mod.rs` — module, CALL_DISPATCH P7, ALL_CATEGORIES 15; 193 lines
- `crates/sdivi-patterns/src/queries/tests.rs` — count 15, M41 tests; 300 lines
- `crates/sdivi-core/src/categories.rs` — http_routing CATALOG_ENTRIES, CATEGORIES[14]; 268 lines
- `crates/sdivi-core/tests/category_contract.rs` — count 15; 279 lines
- `crates/sdivi-core/tests/category_contract_m41.rs` (NEW) — M41 acceptance criterion tests; 154 lines
- `crates/sdivi-patterns/tests/dispatch_disjointness.rs` — http_routing import, all_matching_categories, corpus entries, KNOWN_OVERLAPS, match arm; 299 lines
- `bindings/sdivi-wasm/tests/wasm_smoke.rs` — count 15, http_routing name; 261 lines
- `bindings/sdivi-wasm/tests/m23_native.rs` — count 15 (stale M23 fixture updated), http_routing name; 93 lines
- `docs/pattern-categories.md` — canonical list, TS/JS table, Go table, callee-text section, dispatch table P7, KNOWN_OVERLAPS, active slots, embedder responsibility #13
- `MIGRATION_NOTES.md` — M41 section added (before M40)
- `CHANGELOG.md` — M41 entry added

## Human Notes Status
- Reviewer (M40) noted wasm_smoke.rs coverage gap stale comment: NOT_ADDRESSED (pre-existing, out of scope)
- Reviewer (M40) noted missing blank line at mod.rs:123-124: NOT_ADDRESSED (pre-existing, out of scope)
- Reviewer (M40) noted category_for_node_kind doc only lists logging: NOT_ADDRESSED (pre-existing, out of scope)
- Tester (M40) found 0 bugs: nothing to fix

## Docs Updated
- `docs/pattern-categories.md` — canonical list, TS/JS/Go per-language tables, callee-text section (http_routing::matches_callee), dispatch table P7 active, KNOWN_OVERLAPS, embedder responsibility #13.
- `MIGRATION_NOTES.md` — M41 section (count-introduction, headline before/after example, client/decorator notes, escape hatch).
- `CHANGELOG.md` — M41 entry under `[Unreleased]`.

## Observed Issues (out of scope)
- Pre-existing: `wasm_package_json_version_matches_workspace` test failure — `package.json` stranded at older version (0.2.23 vs 0.2.32). Not introduced by M41.
- Pre-existing: Test name `null_safety_node_kinds_do_not_match_non_ts_js_languages` is semantically inverted (body asserts a match). Carry-over from M37.
- Pre-existing: `crates/sdivi-patterns/src/queries/mod.rs` — no blank line between closing `];` of `CALL_DISPATCH` and the `/// Classify…` doc block for `classify_hint`. Carry-over from M38/M39/M40.
- Pre-existing: `category_for_node_kind` doc comment lists only `logging` as callee-only category; `framework_hooks`, `schema_validation`, `state_store`, `collection_pipelines`, and `http_routing` are also callee-only.
- Fixed (latent bug, now surfaced by M41 WASM count requirement): `bindings/sdivi-wasm/tests/m23_native.rs` had stale count assertion (8 instead of 15) that was drifting since M23. This was broken from at least M34 onward; updated as part of M41 WASM test requirement.
