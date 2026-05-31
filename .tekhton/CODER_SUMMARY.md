# Coder Summary
## Status: COMPLETE

## What Was Implemented

### M37: `null_safety` pattern category (TypeScript / JavaScript)

**`crates/sdivi-patterns/src/queries/null_safety.rs`** (NEW)
- Created with `NODE_KINDS: &[&str] = &["optional_chain", "non_null_expression"]`.
- Module doc explains TS/JS language support, per-node count semantics for
  `optional_chain`, and the deferred `??` nullish coalescing decision.
- Inline unit tests: `node_kinds_contains_optional_chain`, `node_kinds_contains_non_null_expression`, `node_kinds_has_two_entries`.

**`crates/sdivi-patterns/src/queries/mod.rs`**
- Added `pub mod null_safety;` declaration (alphabetical order).
- Added `"null_safety"` to `ALL_CATEGORIES` (10 → 11 entries).
- Added `null_safety` branch to `category_for_node_kind`.
- Updated doc example count assertion 10 → 11.

**`crates/sdivi-core/src/categories.rs`**
- Added `null_safety` entry to `CATALOG_ENTRIES` (between `logging` and `resource_management`).
- Extended `CATEGORIES` constant to index 10 (`CATALOG_ENTRIES[10].0`).
- Updated doc example assertions to include `null_safety` and 11 count.

**`crates/sdivi-lang-typescript/src/extract.rs`**
- Added `"optional_chain"` and `"non_null_expression"` to `PATTERN_KINDS`.

**`crates/sdivi-lang-javascript/src/extract.rs`**
- Added `"optional_chain"` to `PATTERN_KINDS` (`non_null_expression` is TS-only).

**`crates/sdivi-core/tests/category_contract.rs`**
- Updated `list_categories_returns_exactly_ten_categories` → `list_categories_returns_exactly_eleven_categories` (10 → 11).
- Added `list_categories_includes_null_safety` test.
- Added `optional_chain_typescript_is_null_safety` test (M37 acceptance criterion).
- Added `non_null_expression_typescript_is_null_safety` test (M37 acceptance criterion).

**`crates/sdivi-patterns/src/queries/tests.rs`**
- Updated `all_categories_has_ten_entries` → `all_categories_has_eleven_entries` (10 → 11).
- Added `optional_chain_is_null_safety`, `non_null_expression_is_null_safety`, and `null_safety_node_kinds_do_not_match_non_ts_js_languages` tests.

**`crates/sdivi-lang-typescript/tests/extract_behavior.rs`**
- Refactored: M36.1 decorator tests moved to `decorator_hints.rs` (new) to stay under 300-line ceiling.
- Remaining: adapter + import/export + basic pattern hints + class_hierarchy tests (231 lines).

**`crates/sdivi-lang-typescript/tests/decorator_hints.rs`** (NEW — extracted from extract_behavior.rs)
- M36.1 decorator hint tests relocated here (no logic changes). 129 lines.

**`crates/sdivi-lang-typescript/tests/null_safety_hints.rs`** (NEW)
- `optional_chain_captured_as_pattern_hint` — `user?.name` produces hint.
- `non_null_expression_captured_as_pattern_hint` — `el!` produces hint.
- `ts_fixture_with_optional_chain_and_non_null_yields_two_null_safety_instances` — M37 acceptance criterion.
- `optional_chain_member_access_variants_captured` — documents `obj?.field` + `arr?.[0]` both emit `optional_chain`; `fn?.()` does NOT (grammar produces `call_expression` for optional calls).
- `file_with_no_optional_chain_produces_no_null_safety_hints` — negative case.

**`crates/sdivi-lang-javascript/tests/extract_behavior.rs`**
- Added `optional_chain_captured_as_pattern_hint`, `js_has_no_non_null_expression_hints`, `file_with_no_optional_chain_produces_no_null_safety_hints`.

**`docs/pattern-categories.md`**
- Added `null_safety` row to canonical category list table.
- Added `null_safety` row to TypeScript / JavaScript per-language node-kind table.
- Added embedder responsibility #9 describing M37 count-introduction event.
- Renumbered old item 9 → 10.

**`MIGRATION_NOTES.md`**
- Added M37 section before M33: count-introduction event, count semantics, deferred `??`, escape hatch, trend continuity.

**`CHANGELOG.md`**
- Added M37 entry under `[Unreleased] ### Added`.

### Design observation: `fn?.()` does not emit `optional_chain`

The tree-sitter-typescript grammar represents `fn?.()` as a `call_expression` — the optional-call marker is not a named `optional_chain` node. `optional_chain` only appears as a child of `member_expression` (`a?.b`) and `subscript_expression` (`arr?.[0]`). This is consistent with the node-types.json which shows `optional_chain` only in those two field sets. The milestone examples listed `fn?.()` as an `optional_chain` case; this is incorrect for the pinned grammar (v0.21.2). The test documents the actual behavior.

## Root Cause (bugs only)
N/A — feature implementation.

## Files Modified
- `crates/sdivi-patterns/src/queries/null_safety.rs` (NEW) — 69 lines
- `crates/sdivi-patterns/src/queries/mod.rs` — added module, category entry, count update; 177 lines
- `crates/sdivi-patterns/src/queries/tests.rs` — count update + null_safety tests; 172 lines
- `crates/sdivi-core/src/categories.rs` — null_safety CATALOG_ENTRIES, CATEGORIES[10]; 200 lines
- `crates/sdivi-lang-typescript/src/extract.rs` — added optional_chain, non_null_expression to PATTERN_KINDS; 193 lines
- `crates/sdivi-lang-javascript/src/extract.rs` — added optional_chain to PATTERN_KINDS; 208 lines
- `crates/sdivi-lang-typescript/tests/extract_behavior.rs` — extracted decorator tests; 231 lines
- `crates/sdivi-lang-typescript/tests/decorator_hints.rs` (NEW) — relocated M36.1 decorator tests; 129 lines
- `crates/sdivi-lang-typescript/tests/null_safety_hints.rs` (NEW) — M37 TS null_safety tests; 100 lines
- `crates/sdivi-lang-javascript/tests/extract_behavior.rs` — added null_safety hint tests; 263 lines
- `crates/sdivi-core/tests/category_contract.rs` — count update, null_safety acceptance criterion tests; 279 lines
- `docs/pattern-categories.md` — canonical list, TS/JS table, embedder responsibility; doc file
- `MIGRATION_NOTES.md` — M37 section added; doc file
- `CHANGELOG.md` — M37 entry added; doc file

## Human Notes Status
- Stale dispatch comment at `mod.rs:117` ("P1/P6/P8/P9 active at M35"): NOT_ADDRESSED — out of scope; `null_safety` is node-kind-only and does not appear in `CALL_DISPATCH`
- Stale dispatch comment at `dispatch_disjointness.rs:26` ("At M35, P1/P6/P8/P9 are active"): NOT_ADDRESSED — out of scope per task scope rules

## Docs Updated
- `docs/pattern-categories.md` — null_safety row in canonical list, TS/JS table, embedder #9 added.
- `MIGRATION_NOTES.md` — M37 section (count-introduction event, deferred `??`, escape hatch).
- `CHANGELOG.md` — M37 entry under `[Unreleased]`.

## Observed Issues (out of scope)
- Pre-existing: `wasm_package_json_version_matches_workspace` test failure — `package.json` stranded at `0.2.23`, workspace now at `0.2.28`. Not introduced by M37.
- Pre-existing: `dispatch_disjointness.rs:26` comment "At M35, P1/P6/P8/P9 are active" is stale (now M37).
