# Coder Summary
## Status: COMPLETE

## What Was Implemented

### M36.1: `decorators` pattern category (TS/JS)

**`crates/sdivi-lang-typescript/src/extract.rs`**
- Added `"decorator"` to `PATTERN_KINDS` — the parsing stage now emits `decorator` nodes
  as `PatternHint` values (previously uncollected).

**`crates/sdivi-lang-javascript/src/extract.rs`**
- Added `"decorator"` to `PATTERN_KINDS` — same change for JS (Stage-3 decorators).

**`crates/sdivi-patterns/src/queries/decorators.rs`** (NEW)
- `NODE_KINDS: &[&str] = &["decorator"]` — node-kind-only detection, no `matches_callee`.
- Doc comment with `# Examples` block (doc test runs in CI).
- Inline unit tests: `node_kinds_contains_decorator`, `node_kinds_has_exactly_one_entry`.

**`crates/sdivi-patterns/src/queries/mod.rs`**
- Added `pub mod decorators;` (alphabetical).
- Added `"decorators"` to `ALL_CATEGORIES` (between `"data_access"` and `"error_handling"`).
- Added `decorators::NODE_KINDS.contains(&node_kind) → Some("decorators")` branch to
  `category_for_node_kind` (after `data_access`, before `error_handling`).
- Updated doc example: count 9 → 10, added `decorators` assertion.
- Updated inline test: renamed to `all_categories_has_ten_entries`, added `decorators` assert.
- Added `decorator_is_decorators` test (acceptance criterion: returns `Some("decorators")`).
- Shortened `Note:` paragraph and `# See also` section to stay at 299 lines.

**`crates/sdivi-core/src/categories.rs`**
- Added `"decorators"` entry to `CATALOG_ENTRIES` (index 3, alphabetical).
- Added `CATALOG_ENTRIES[9].0` to `CATEGORIES` const (now 10 entries).
- Updated doc example: count 9 → 10, changed assertions to reference `"decorators"`.

**`crates/sdivi-core/tests/category_contract.rs`**
- Renamed `list_categories_returns_exactly_nine_categories` → `..._ten_categories`, count 9 → 10.
- Added `list_categories_includes_decorators` test.

**`crates/sdivi-patterns/tests/dispatch_disjointness.rs`**
- Added a CORPUS entry documenting that `@Injectable()` routed as `call_expression`
  produces `""` — decorators are node-kind-only and bypass `CALL_DISPATCH`.

**`bindings/sdivi-wasm/tests/wasm_smoke.rs`**
- Updated count 9 → 10, added `names.contains(&"decorators")` assertion.

**`docs/pattern-categories.md`**
- Added `decorators` row to canonical category list table.
- Added `decorator` row to TypeScript/JavaScript node-kind mapping table.
- Updated dispatch order note to clarify `decorators` is node-kind-only (not in `CALL_DISPATCH`).
- Updated KNOWN_OVERLAPS section header: "at M34" → "at M35".
- Added embedder responsibility item #8 for `decorators`.

**`MIGRATION_NOTES.md`**
- Added M36.1 section: count-introduction event, parsing-layer change note, escape hatch,
  trend continuity.

**`CHANGELOG.md`**
- Added `decorators` entry under `[Unreleased] ### Added`.

## Root Cause (bugs only)
N/A — additive feature work.

## Files Modified
- `crates/sdivi-lang-typescript/src/extract.rs` — added `"decorator"` to `PATTERN_KINDS`
- `crates/sdivi-lang-javascript/src/extract.rs` — added `"decorator"` to `PATTERN_KINDS`
- `crates/sdivi-patterns/src/queries/decorators.rs` (NEW) — 39 lines
- `crates/sdivi-patterns/src/queries/mod.rs` — 299 lines
- `crates/sdivi-core/src/categories.rs` — 188 lines
- `crates/sdivi-core/tests/category_contract.rs` — 235 lines
- `crates/sdivi-patterns/tests/dispatch_disjointness.rs` — 206 lines
- `bindings/sdivi-wasm/tests/wasm_smoke.rs` — 253 lines
- `docs/pattern-categories.md` — canonical list, TS/JS table, dispatch note, embedder #8
- `MIGRATION_NOTES.md` — M36.1 section added
- `CHANGELOG.md` — `decorators` entry added

## Human Notes Status
No Human Notes section present in this milestone.

## Docs Updated
- `docs/pattern-categories.md` — `decorators` added to canonical category list table,
  TS/JS node-kind table, dispatch order note updated, embedder responsibilities updated.
- `MIGRATION_NOTES.md` — M36.1 section added (count-introduction event, parsing-layer change).
- `CHANGELOG.md` — Added entry under `[Unreleased]`.

## Observed Issues (out of scope)
- Pre-existing: `wasm_package_json_version_matches_workspace` test failure — `package.json`
  stranded at `0.2.23`, workspace at `0.2.26`. Not introduced by M36.1.
- Pre-existing: stale assertion message at `crates/sdivi-patterns/src/queries/mod.rs:282`
  ("logging is catalog-only in v0 for category_for_node_kind") — previously flagged in M23
  and M34 reviewer reports.
- `docs/pattern-categories.md` KNOWN_OVERLAPS section header previously read
  "at M34 (P1/P8/P9 active)" — updated to "at M35" as part of this milestone's doc work
  (this was the pre-existing cosmetic note from the M35 reviewer).
