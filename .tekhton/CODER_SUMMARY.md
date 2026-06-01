# Coder Summary
## Status: COMPLETE

## What Was Implemented

### M46: Pattern Category — `comprehensions` (Python-only)

**`crates/sdivi-patterns/src/queries/comprehensions.rs`** (NEW) — 77 lines
- `NODE_KINDS: &[&str]` with four entries: `dictionary_comprehension`,
  `generator_expression`, `list_comprehension`, `set_comprehension`.
- Inline `#[cfg(test)] mod tests` with 7 unit tests covering each kind and a
  non-Python sanity check.

**`crates/sdivi-patterns/src/queries/mod.rs`** — 207 lines
- Added `pub mod comprehensions;` (alphabetically between `collection_pipelines`
  and `concurrency`).
- Added `"comprehensions"` to `ALL_CATEGORIES` in alphabetical order.
- Added comprehensions arm to `category_for_node_kind` (between class_hierarchy
  and concurrency branches).
- Updated `ALL_CATEGORIES` doc count from 18 to 19.

**`crates/sdivi-core/src/categories.rs`** — 299 lines
- Inserted comprehensions entry in `CATALOG_ENTRIES` (position 3, between
  `collection_pipelines` and `concurrency`).
- Added `CATALOG_ENTRIES[18].0` to `CATEGORIES` constant (now 19 entries).
- Updated doc examples to reference `comprehensions` and count 19.

**`crates/sdivi-core/tests/category_contract_m46.rs`** (NEW) — 124 lines
- M46 acceptance-criterion tests: all four node kinds classify to comprehensions,
  classify_hint routing, non-match for unknown nodes, count = 19, includes
  "comprehensions" in list_categories().

**Count tests updated (my changes broke these — fixed):**
- `category_contract.rs` — `list_categories_returns_exactly_eighteen_categories`
  renamed and updated to assert 19.
- `category_contract_m42.rs` — count assertion updated to 19.
- `category_contract_m43.rs` — count assertion updated to 19.
- `category_contract_m44.rs` — count assertion updated to 19.
- `category_contract_m45_1.rs` — count assertion updated to 19.
- `category_contract_m45_2.rs` — count assertion updated to 19.
- `crates/sdivi-patterns/src/queries/tests.rs` — `all_categories_has_eighteen_entries`
  renamed and updated to assert 19.

**`docs/pattern-categories.md`**
- Canonical list table: added `comprehensions` row (between `collection_pipelines`
  and `data_access` in the doc table order).
- Python table: added `comprehensions` row with all four node kinds.

**`MIGRATION_NOTES.md`**
- Added M46 section before M45.2: schema unchanged, count 18→19, additive-only,
  count-introduction event, nested comprehension counting, escape hatch TOML.

**`CHANGELOG.md`**
- Added M46 entry under `[Unreleased] ### Added` before M45.2 entry.

## Verification
- Python adapter already collects all four node kinds (`PATTERN_KINDS` in
  `crates/sdivi-lang-python/src/extract.rs`) — no parsing change required.
- `cargo test --workspace`: 1444+ tests pass. Only pre-existing failure:
  `wasm_package_json_version_matches_workspace` (package.json at 0.2.23 vs
  workspace 0.2.38; not introduced by M46).

## Root Cause (bugs only)
N/A — feature addition.

## Files Modified
- `crates/sdivi-patterns/src/queries/comprehensions.rs` (NEW) — 77 lines
- `crates/sdivi-patterns/src/queries/mod.rs` — pub mod, ALL_CATEGORIES, category_for_node_kind
- `crates/sdivi-core/src/categories.rs` — CATALOG_ENTRIES + CATEGORIES updated; 299 lines
- `crates/sdivi-core/tests/category_contract_m46.rs` (NEW) — 124 lines
- `crates/sdivi-core/tests/category_contract.rs` — count 18 → 19
- `crates/sdivi-core/tests/category_contract_m42.rs` — count 18 → 19
- `crates/sdivi-core/tests/category_contract_m43.rs` — count 18 → 19
- `crates/sdivi-core/tests/category_contract_m44.rs` — count 18 → 19
- `crates/sdivi-core/tests/category_contract_m45_1.rs` — count 18 → 19
- `crates/sdivi-core/tests/category_contract_m45_2.rs` — count 18 → 19
- `crates/sdivi-patterns/src/queries/tests.rs` — count 18 → 19
- `docs/pattern-categories.md` — canonical list + Python table
- `MIGRATION_NOTES.md` — M46 section
- `CHANGELOG.md` — M46 entry

## Human Notes Status
No human notes in this run.

## Docs Updated
- `docs/pattern-categories.md` — canonical category list (comprehensions row),
  Python per-language table (comprehensions row).
- `MIGRATION_NOTES.md` — M46 section.
- `CHANGELOG.md` — M46 entry.

## Observed Issues (out of scope)
- Pre-existing: `wasm_package_json_version_matches_workspace` — package.json
  stranded at 0.2.23 vs workspace 0.2.38.
- Pre-existing: `ALL_CATEGORIES` doc note in `mod.rs` says only `logging` is
  callee-only via `classify_hint`; several other categories are also callee-only.
- Pre-existing: `docs/pattern-categories.md` embedder responsibilities list has
  a numbering regression across M42–M44.
