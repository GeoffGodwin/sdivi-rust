# Coder Summary
## Status: COMPLETE

## What Was Implemented

### M36.2: `decorators` pattern category extended to Python

**`crates/sdivi-patterns/src/queries/decorators.rs`**
- Added `"decorated_definition"` to `NODE_KINDS` (now 2 entries: `["decorator", "decorated_definition"]`).
- Updated module doc comment to explain Python support and the wrapper-granularity count semantics.
- Updated `NODE_KINDS` doc comment to document both node kinds.
- Updated inline unit test: `node_kinds_has_exactly_one_entry` → `node_kinds_has_two_entries`.
- Added `node_kinds_contains_decorated_definition` test.

**`crates/sdivi-patterns/src/queries/tests.rs`** (NEW — extracted from mod.rs)
- Extracted the entire `#[cfg(test)] mod tests { ... }` block from `mod.rs` into a
  separate `tests.rs` file. Required because applying `cargo fmt` to the pre-existing
  long-line assertions in `mod.rs` pushed it from 299 → 311 lines, and extracting the
  tests block brings `mod.rs` back to 172 lines. No test logic was changed.

**`crates/sdivi-patterns/src/queries/mod.rs`**
- Applied `cargo fmt` to resolve pre-existing long-line formatting violations (M36.1
  introduced assertions that exceeded rustfmt's line length).
- Replaced the inline `mod tests { ... }` block with `#[cfg(test)] mod tests;` (pointing
  to the new `tests.rs` file).

**`crates/sdivi-lang-python/tests/extract_behavior.rs`**
- Added `decorated_definition_captured_as_decorator_hint` — unit test for `@dataclass` class.
- Added `fastapi_and_pytest_fixture_produce_decorated_definition_hints` — integration test:
  `@app.get` + `@pytest.fixture` → 2 `decorated_definition` hints (acceptance criterion).
- Added `stacked_decorators_count_as_one_decorated_definition` — documents wrapper-granularity.
- Added `file_with_no_decorators_produces_no_decorated_definition_hints` — negative case.

**`crates/sdivi-core/tests/category_contract.rs`**
- Added `decorated_definition_python_is_decorators` test (acceptance criterion:
  `category_for_node_kind("decorated_definition", "python") == Some("decorators")`).

**`docs/pattern-categories.md`**
- Canonical category list: updated `decorators` row to describe Python support and count semantics.
- Python table: added `decorators` row with `decorated_definition` and wrapper-granularity note.
- Line 185 fix (M36.1 reviewer note): "P10 (`decorators`) is node-kind-only…" →
  "The `decorators` category is node-kind-only…"; milestone marker updated to M36.2.
- Embedder responsibility #8: extended to describe Python M36.2 support and count asymmetry.

**`MIGRATION_NOTES.md`**
- Added M36.2 section before M36.1 section: count-introduction event, wrapper-granularity
  semantics, cross-language asymmetry note, escape hatch, trend continuity.

**`CHANGELOG.md`**
- Added M36.2 entry under `[Unreleased] ### Added`.

## Root Cause (bugs only)
N/A — additive feature work.

## Files Modified
- `crates/sdivi-patterns/src/queries/decorators.rs` — added `decorated_definition`, updated docs/tests (58 lines)
- `crates/sdivi-patterns/src/queries/tests.rs` (NEW) — extracted from mod.rs inline tests block (138 lines)
- `crates/sdivi-patterns/src/queries/mod.rs` — applied fmt, replaced inline tests with `mod tests;` (172 lines)
- `crates/sdivi-lang-python/tests/extract_behavior.rs` — added 4 decorator hint tests (298 lines)
- `crates/sdivi-core/tests/category_contract.rs` — added `decorated_definition_python_is_decorators` test (247 lines)
- `docs/pattern-categories.md` — Python table, canonical list, P10 label fix, embedder #8 (270 lines)
- `MIGRATION_NOTES.md` — M36.2 section added (353 lines — doc file, not subject to code ceiling)
- `CHANGELOG.md` — M36.2 entry added (720 lines — doc file, not subject to code ceiling)

## Human Notes Status
Non-Blocking Notes from M36.1 reviewer:
- "P10 (`decorators`)" label in docs/pattern-categories.md — COMPLETED: fixed to "The `decorators` category"
- `dispatch_disjointness.rs:26` stale comment — NOT_ADDRESSED: out of scope for M36.2 (decorators still adds no CALL_DISPATCH entry)
- Double-counting risk note — NOT_ADDRESSED: pre-existing observation, no action required
- Pre-existing stale assertion message at mod.rs:282 — NOT_ADDRESSED: message content unchanged (comment is still accurate)
- Pre-existing WASM package.json version — NOT_ADDRESSED: pre-existing from before M36.1

## Docs Updated
- `docs/pattern-categories.md` — Python node-kind table row, canonical list updated, P10 label fixed, embedder #8 extended.
- `MIGRATION_NOTES.md` — M36.2 section added (wrapper-granularity semantics, count asymmetry, escape hatch).
- `CHANGELOG.md` — M36.2 entry added under `[Unreleased]`.

## Observed Issues (out of scope)
- Pre-existing: `wasm_package_json_version_matches_workspace` test failure — `package.json` stranded at `0.2.23`, workspace at `0.2.27`. Not introduced by M36.2.
- Pre-existing: `dispatch_disjointness.rs:26` comment "At M35, P1/P6/P8/P9 are active" is stale (now M36.2); flagged by M36.1 reviewer.
- Pre-existing: `docs/pattern-categories.md` Go corpus — `fmt.Errorf` classified as `logging` (constructs an error value, not output); inherited from M33.
