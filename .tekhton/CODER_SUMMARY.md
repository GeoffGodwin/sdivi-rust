# Coder Summary
## Status: COMPLETE

## What Was Implemented

### M42: `testing` pattern category

**`crates/sdivi-patterns/src/queries/testing.rs`** (NEW — 246 lines)
- `NODE_KINDS: &[&str] = &[]` (call-based; no node-kind matching).
- Four `LazyLock<Regex>` statics: `TS_JS_GLOBALS_RE` (BDD globals + hooks + expect),
  `TS_JS_FRAMEWORK_RE` (jest/vi helpers), `GO_RE` (`\bt\.` word-boundary anchored),
  `PYTHON_RE` (`self.assert[A-Z]…`).
- `matches_callee(text, language)` — TypeScript/JavaScript, Go, Python; all others
  return `false`.
- Module doc covers: CALL_DISPATCH slot P2, false-positive policy, `scope_exclude`
  interaction, seeds forward (property-based / E2E).
- Inline unit tests: all TS/JS globals, jest/vi helpers, Go positives/negatives
  (including `st.Run` non-match), Python assertion methods, language fallthrough.

**`crates/sdivi-patterns/src/queries/mod.rs`**
- Added `pub mod testing;` (alphabetical, between `state_store` and `type_assertions`).
- Added `"testing"` to `ALL_CATEGORIES` (15 → 16 entries); updated doc example.
- Inserted `("testing", testing::matches_callee)` at P2 in `CALL_DISPATCH` (after
  `async_patterns` P1, before `schema_validation` P4).
- Updated dispatch comment: P2=testing added.
- Updated `classify_hint` doc: M41 → M42 active-slots reference.

**`crates/sdivi-core/src/categories.rs`**
- Added `testing` entry to `CATALOG_ENTRIES` (alphabetically between `state_store`
  and `type_assertions`), with full description including `scope_exclude` note.
- Added `CATALOG_ENTRIES[14].0` to `CATEGORIES` (15 → 16 entries).
- Updated `CATEGORIES` and `list_categories` doc examples: 15 → 16, added `testing`.

**`crates/sdivi-patterns/src/queries/tests.rs`**
- Renamed `all_categories_has_fifteen_entries` → `all_categories_has_sixteen_entries`.
- Updated count 15 → 16; swapped `null_safety` assertion for `testing` (null_safety
  is covered extensively in its own section below).

**`crates/sdivi-core/tests/category_contract.rs`**
- Renamed count test: 15 → 16 (`list_categories_returns_exactly_sixteen_categories`).

**`crates/sdivi-core/tests/category_contract_m42.rs`** (NEW — 161 lines)
- M42 acceptance-criterion tests via `classify_hint`:
  - `expect(x).toBe(1)` → `["testing"]` (milestone criterion).
  - `describe('s', fn)` → `["testing"]` (milestone criterion).
  - Full TS/JS, Go, Python positives.
  - Negatives: console.log → logging, axios.get → data_access, useEffect →
    framework_hooks, self.method() → not testing.
  - `list_categories_includes_testing`, `list_categories_count_is_sixteen`.
  - 18 tests total.

**`crates/sdivi-patterns/tests/dispatch_disjointness.rs`**
- Added `testing` to use imports (inline; no new line).
- Added `testing::matches_callee` block in `all_matching_categories`; updated M41 → M42
  comment.
- Added one corpus entry: `("describe('suite', fn)", "typescript", "testing")`.
- Added `"testing" => testing::matches_callee(text, lang)` to match arm.
- Removed 5 redundant section-comment lines to keep the file at exactly 300 lines.

**`bindings/sdivi-wasm/tests/wasm_smoke.rs`**
- Updated count 15 → 16; added `assert!(names.contains(&"testing"))`.

**`bindings/sdivi-wasm/tests/m23_native.rs`**
- Updated count 15 → 16; added `testing` name assertion.

**`docs/pattern-categories.md`**
- Added `testing` row to canonical category list table.
- Added `testing` row to TypeScript/JavaScript per-language table.
- Added `testing` row to Go/Java per-language table.
- Added `testing::matches_callee` callee-text section (after `async_patterns`).
- Updated dispatch table P2: active **M42** (was a future placeholder).
- Updated active-slots comment: M41 → M42.
- Added embedder responsibility #14 for M42; renumbered old #14 → #15.

**`MIGRATION_NOTES.md`**
- Added M42 section (before M41): count-introduction event (15→16), `scope_exclude`
  interaction with example TOML, false-positive note, escape hatch.

**`CHANGELOG.md`**
- Added M42 entry under `[Unreleased] ### Added` (before M41 entry).

## Root Cause (bugs only)
N/A — new feature milestone.

## Files Modified
- `crates/sdivi-patterns/src/queries/testing.rs` (NEW) — 246 lines
- `crates/sdivi-patterns/src/queries/mod.rs` — module, CALL_DISPATCH P2, ALL_CATEGORIES 16; 197 lines
- `crates/sdivi-core/src/categories.rs` — testing CATALOG_ENTRIES, CATEGORIES[14]; 287 lines
- `crates/sdivi-patterns/src/queries/tests.rs` — count 16, testing assertion; 300 lines
- `crates/sdivi-core/tests/category_contract.rs` — count 16; 279 lines
- `crates/sdivi-core/tests/category_contract_m42.rs` (NEW) — M42 acceptance criterion tests; 161 lines
- `crates/sdivi-patterns/tests/dispatch_disjointness.rs` — testing import, all_matching_categories, corpus entry, match arm; 300 lines
- `bindings/sdivi-wasm/tests/wasm_smoke.rs` — count 16, testing name; 262 lines
- `bindings/sdivi-wasm/tests/m23_native.rs` — count 16, testing name; 97 lines
- `docs/pattern-categories.md` — canonical list, TS/JS/Go tables, callee-text section, dispatch table P2, embedder responsibility #14
- `MIGRATION_NOTES.md` — M42 section added (before M41)
- `CHANGELOG.md` — M42 entry added

## Human Notes Status
No human notes in this task. Prior-run reviewer notes (M41) were pre-existing out-of-scope.

## Docs Updated
- `docs/pattern-categories.md` — canonical list table, TS/JS table, Go table, callee-text
  section (`testing::matches_callee`), dispatch table P2 active, embedder responsibility #14.
- `MIGRATION_NOTES.md` — M42 section with `scope_exclude` interaction and escape hatch.
- `CHANGELOG.md` — M42 entry under `[Unreleased]`.

## Observed Issues (out of scope)
- Pre-existing: `wasm_package_json_version_matches_workspace` test failure — `package.json`
  stranded at 0.2.23 vs workspace 0.2.33. Not introduced by M42.
- Pre-existing: Test name `null_safety_node_kinds_do_not_match_non_ts_js_languages` is
  semantically inverted (body asserts a match). Carry-over from M37.
- Pre-existing: `crates/sdivi-patterns/src/queries/mod.rs` — no blank line between closing
  `];` of `CALL_DISPATCH` and the `/// Classify…` doc block for `classify_hint`. Carry-over.
- Pre-existing: `category_for_node_kind` doc comment lists only `logging` as callee-only;
  `framework_hooks`, `schema_validation`, `state_store`, `collection_pipelines`, `http_routing`,
  and `testing` are also callee-only.
