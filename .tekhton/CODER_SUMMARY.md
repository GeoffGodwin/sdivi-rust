# Coder Summary
## Status: COMPLETE

## What Was Implemented

### M45.2: Enrich `error_handling` with Python/Java clause and throw-site node kinds

**`crates/sdivi-patterns/src/queries/error_handling.rs`** — 21 lines
- Extended `NODE_KINDS` from 2 to 6 entries: added `try_statement` (pre-existing per
  docs/adapters but previously missing from NODE_KINDS — doc/code drift resolved as part
  of this milestone's "existing" assumption), `except_clause` (Python), `catch_clause`
  and `throw_statement` (Java).
- Updated doc comment to document all six forms.

**`crates/sdivi-patterns/src/queries/tests_m45_2.rs`** (NEW) — 41 lines
- Four unit tests: `try_statement_is_error_handling`, `except_clause_is_error_handling`,
  `catch_clause_is_error_handling`, `throw_statement_is_error_handling`.
- Declared via `#[cfg(test)] mod tests_m45_2;` in `queries/mod.rs` — split from `tests.rs`
  to keep that file at exactly 300 lines.

**`crates/sdivi-patterns/src/queries/mod.rs`** — `#[cfg(test)] mod tests_m45_2;` added.

**`crates/sdivi-core/tests/category_contract_m45_2.rs`** (NEW) — 110 lines
- M45.2 acceptance-criterion tests: all three new node kinds classify to error_handling,
  classify_hint routing verified, existing Rust kinds unchanged, list_categories count = 18.

**`crates/sdivi-patterns/tests/error_handling_fixture.rs`** (NEW) — 179 lines
- Integration tests via `build_catalog`:
  - `python_except_clause_routes_to_error_handling`
  - `python_multi_arm_except_counts_each_clause` — 1 try_statement + 3 except_clause = 4
    (double-count semantic verified)
  - `java_catch_clause_routes_to_error_handling`
  - `java_throw_statement_routes_to_error_handling`
  - `java_mixed_error_handling_counts` — 2 catch + 1 throw = 3

**`docs/pattern-categories.md`**
- Canonical list: updated `error_handling` description to include Python `except_clause`
  and Java `catch_clause`/`throw_statement` with double-count semantic note.
- Python table: updated `error_handling` row to include `except_clause` with per-arm
  counting note.
- Go/Java table: added `error_handling` row with `catch_clause`, `throw_statement`
  (Java only; Go: none in v0).

**`MIGRATION_NOTES.md`**
- Added M45.2 section before M45.1: schema unchanged, double-count semantic documented,
  count-introduction event, escape hatch TOML.

**`CHANGELOG.md`**
- Added M45.2 entry under `[Unreleased] ### Added` before M45.1 entry.

## Root Cause (bugs only)
N/A — feature addition.

## Files Modified
- `crates/sdivi-patterns/src/queries/error_handling.rs` — 6 node kinds (was 2); 21 lines
- `crates/sdivi-patterns/src/queries/mod.rs` — added `#[cfg(test)] mod tests_m45_2;`
- `crates/sdivi-patterns/src/queries/tests_m45_2.rs` (NEW) — 4 unit tests; 41 lines
- `crates/sdivi-core/tests/category_contract_m45_2.rs` (NEW) — acceptance criteria; 110 lines
- `crates/sdivi-patterns/tests/error_handling_fixture.rs` (NEW) — integration fixtures; 179 lines
- `docs/pattern-categories.md` — canonical list, Python table, Go/Java table
- `MIGRATION_NOTES.md` — M45.2 section
- `CHANGELOG.md` — M45.2 entry

## Human Notes Status
- Non-blocking note (cargo test before merge): done — all tests pass except pre-existing
  `wasm_package_json_version_matches_workspace` (0.2.23 vs 0.2.37, not introduced here).

## Docs Updated
- `docs/pattern-categories.md` — canonical `error_handling` description, Python table
  (except_clause added), Go/Java table (new error_handling row).
- `MIGRATION_NOTES.md` — M45.2 section with double-count semantic, count-introduction
  event, escape hatch.
- `CHANGELOG.md` — M45.2 entry.

## Observed Issues (out of scope)
- Pre-existing: `wasm_package_json_version_matches_workspace` — package.json stranded at
  0.2.23 vs workspace 0.2.37.
- Pre-existing: `ALL_CATEGORIES` doc note in `mod.rs:36-37` says only `logging` is
  callee-only via `classify_hint`; several other categories are also callee-only.
- Pre-existing: `docs/pattern-categories.md` embedder responsibilities list has a
  numbering regression across M42–M44.
- Pre-existing (resolved in scope): `try_statement` was documented as existing in
  `error_handling` NODE_KINDS but was missing from the code — added to align with the
  milestone's "existing" assertion and the docs/adapters.
