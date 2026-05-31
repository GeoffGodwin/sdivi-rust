# Coder Summary
## Status: COMPLETE

## What Was Implemented

### M38: `schema_validation` pattern category

**`crates/sdivi-patterns/src/queries/schema_validation.rs`** (NEW)
- Created with `NODE_KINDS: &[&str] = &[]` (call-based, no node-kind matching).
- `LazyLock<Regex>` tables: `TS_JS_RE` (`^(z|yup|v|s)\.\w|\.safeParse\(`) and `PYTHON_RE` (`\bField\(|\bconstr\(|\bconint\(`).
- `matches_callee(text, language)` — TypeScript/JavaScript and Python only; other languages return `false`.
- Module doc covers: language support, precision-over-recall rationale, known recall gap (arbitrary receiver), Pydantic class vs. call distinction, decorator cross-reference.
- Inline unit tests: Zod/Yup/Valibot/Superstruct positives, `.safeParse(` positive, Pydantic field positives, bare-method negatives, wrong-language negatives, `node_kinds_is_empty`.

**`crates/sdivi-patterns/src/queries/mod.rs`**
- Added `pub mod schema_validation;` (alphabetical order, between `resource_management` and `state_management`).
- Added `"schema_validation"` to `ALL_CATEGORIES` (11 → 12 entries).
- Added `schema_validation` to `CALL_DISPATCH` at slot P4 (between `async_patterns` and `framework_hooks`).
- Updated dispatch order comment: "P1/P4/P6/P8/P9 active at M38".
- Updated doc example count assertion 11 → 12.

**`crates/sdivi-core/src/categories.rs`**
- Added `schema_validation` entry to `CATALOG_ENTRIES` (between `resource_management` and `state_management`).
- Extended `CATEGORIES` constant to index 11 (`CATALOG_ENTRIES[11].0`).
- Updated doc example assertions to include `schema_validation` and 12 count.

**`crates/sdivi-patterns/src/queries/tests.rs`**
- Added `use crate::hint_input::PatternHintInput;` import.
- Renamed `all_categories_has_eleven_entries` → `all_categories_has_twelve_entries` (11 → 12).
- Added M38 section: `zod_call_expression_is_schema_validation`, `math_max_is_not_schema_validation`.

**`crates/sdivi-core/tests/category_contract.rs`**
- Renamed count test: `list_categories_returns_exactly_eleven_categories` → `list_categories_returns_exactly_twelve_categories` (11 → 12).

**`crates/sdivi-core/tests/category_contract_m38.rs`** (NEW)
- M38 acceptance criterion tests: `zod_object_typescript_is_schema_validation` (`z.object({}) → ["schema_validation"]`), `math_max_returns_empty_not_schema_validation` (`Math.max(a,b) → []`).
- Zod/Yup/Valibot/Superstruct positives; Python Pydantic positives.
- Negatives: bare method calls, wrong languages.
- Disjointness assertion: `.safeParse(` resolves to `schema_validation`, not `data_access`.

**`crates/sdivi-patterns/tests/dispatch_disjointness.rs`**
- Added `schema_validation` to import list.
- Added `schema_validation::matches_callee` to `all_matching_categories` (updated comment from M35 → M38).
- Added 8 corpus entries for schema_validation positives + 1 negative.
- Added `"schema_validation"` arm to `KNOWN_OVERLAPS` loser match.

**`bindings/sdivi-wasm/tests/wasm_smoke.rs`**
- Updated `list_categories_returns_schema_version_and_expected_count`: count 10 → 12, added `null_safety` and `schema_validation` to names assertions (the stale M37 count also corrected here).

**`docs/pattern-categories.md`**
- Added `schema_validation` row to canonical category list table.
- Added `schema_validation` row to Python per-language table.
- Added `schema_validation` row to TypeScript/JavaScript per-language table.
- Added `schema_validation::matches_callee` callee-text classification section (between `async_patterns` and `framework_hooks`).
- Updated "P1/P6/P8/P9 active at M36.2" → "P1/P4/P6/P8/P9 active at M38".
- Updated KNOWN_OVERLAPS heading from M35 → M38.
- Added embedder responsibility #10 for M38 count-introduction event; renumbered old #10 → #11.

**`MIGRATION_NOTES.md`**
- Added M38 section before M37: count-introduction event, library coverage, precision-over-recall, Pydantic class vs. call, decorator cross-reference, escape hatch, trend continuity.

**`CHANGELOG.md`**
- Added M38 entry under `[Unreleased] ### Added` before M37 entry.

## Root Cause (bugs only)
N/A — feature implementation.

## Files Modified
- `crates/sdivi-patterns/src/queries/schema_validation.rs` (NEW) — 176 lines
- `crates/sdivi-patterns/src/queries/mod.rs` — module, CALL_DISPATCH P4, ALL_CATEGORIES 12; 181 lines
- `crates/sdivi-patterns/src/queries/tests.rs` — count 12, M38 tests; 202 lines
- `crates/sdivi-core/src/categories.rs` — schema_validation CATALOG_ENTRIES, CATEGORIES[11]; 215 lines
- `crates/sdivi-core/tests/category_contract.rs` — count 12; 279 lines
- `crates/sdivi-core/tests/category_contract_m38.rs` (NEW) — M38 acceptance criterion tests; 144 lines
- `crates/sdivi-patterns/tests/dispatch_disjointness.rs` — schema_validation import, all_matching_categories, corpus entries; 225 lines
- `bindings/sdivi-wasm/tests/wasm_smoke.rs` — count 12, null_safety + schema_validation names check; 255 lines
- `docs/pattern-categories.md` — canonical list, Python/TS/JS tables, callee-text section, embedder responsibility; doc file
- `MIGRATION_NOTES.md` — M38 section added; doc file
- `CHANGELOG.md` — M38 entry added; doc file

## Human Notes Status
No Human Notes section present in this task.

## Docs Updated
- `docs/pattern-categories.md` — schema_validation in canonical list, Python/TS/JS per-language tables, callee-text section, M38 embedder responsibility, dispatch order comment.
- `MIGRATION_NOTES.md` — M38 section (count-introduction event, library coverage, precision-over-recall, escape hatch).
- `CHANGELOG.md` — M38 entry under `[Unreleased]`.

## Observed Issues (out of scope)
- Pre-existing: `wasm_package_json_version_matches_workspace` test failure — `package.json` stranded at `0.2.23`, workspace at `0.2.29`. Not introduced by M38.
- Pre-existing: Reviewer noted `fn?.()` example appears in M37 docs (`null_safety.rs`, `categories.rs`, `docs/pattern-categories.md`, `MIGRATION_NOTES.md`) but the grammar emits `call_expression` for optional calls, not `optional_chain`. Doc is accurate to the spec but inconsistent with grammar reality — not in M38 scope.
- Pre-existing: Test name `null_safety_node_kinds_do_not_match_non_ts_js_languages` is semantically inverted (body asserts a match). Pre-existing from M37.
