# Coder Summary
## Status: COMPLETE

## What Was Implemented

### M33: Native Pipeline Switchover to `classify_hint`

- **`crates/sdivi-patterns/src/catalog.rs`** — Core behavioral change: replaced `category_for_node_kind` (node-kind-only) with `classify_hint` (node-kind + callee-text). The loop now constructs a `PatternHintInput` from each `PatternHint`, calls `queries::classify_hint`, and fans out into all returned categories. `PatternLocation` is cloned inside the inner loop (correct for the disjoint-regex invariant; cold path in v0). Empty `Vec` return silently drops the hint (same as prior `None` path).

- **`crates/sdivi-patterns/src/queries/mod.rs`** — Strengthened rustdoc on `category_for_node_kind` ("no longer called directly by the native pipeline since M33"). Updated M30 sentinel comment to cross-reference M33. Added reference to `tests/m33_sentinels.rs` for M33 positive sentinels.

- **`crates/sdivi-patterns/tests/m33_sentinels.rs`** (NEW) — Three M33 load-bearing invariant tests:
  - `classify_hint_returns_logging_for_console_log` (TypeScript positive sentinel)
  - `classify_hint_returns_logging_for_tracing_macro` (Rust positive sentinel)
  - `classify_hint_drops_unrecognised_calls` (negative/drop sentinel)

- **`tests/fixtures/simple-typescript/app.ts`** — Extended with `console.log("Starting run")` (→ `logging`) and `fetch(...)` (→ `data_access`) calls. Satisfies the acceptance criterion "logging bucket is non-empty post-M33."

- **`tests/fixtures/simple-rust/src/lib.rs`** — Added 3 `tracing::info!/debug!/trace!` calls so the pipeline produces ≥5 matching `macro_invocation` nodes for the `logging` bucket to survive the default `min_pattern_nodes = 5` filter.

- **`tests/fixtures/simple-rust/src/utils.rs`** — Added 2 `tracing::info!/debug!` calls.

- **`tests/fixtures/simple-python/main.py`** — Added `requests.get(url)` (→ `data_access`) and `logging.info(...)` (→ `logging`) to restore the `data_access` bucket that now requires callee matching.

- **`crates/sdivi-patterns/tests/logging_fixture.rs`** — Rewrote from M30 negative sentinel (no logging bucket) to M33 positive sentinels: `simple_typescript_fixture_produces_logging_bucket_after_m33` and `simple_typescript_fixture_data_access_requires_matching_callee`.

- **`crates/sdivi-pipeline/tests/snapshot_m32_unchanged.rs`** — Replaced `m32_pipeline_snapshot_has_no_logging_entry_in_catalog` (which asserted logging absent) with `m33_pipeline_snapshot_has_logging_entry_for_tracing_macros` (asserts logging present). Determinism and schema-version tests unchanged.

- **`docs/pattern-categories.md`** — Updated canonical category list (data_access narrower, resource_management excludes logging macros, async_patterns adds Promise chains). Per-language tables updated for all 5 language groups: Rust, Python, TypeScript/JavaScript, Go/Java. Updated "Callee-text classification" section header to note regex tables are now load-bearing for native output. Updated embedder responsibilities §5 and §6 to reflect M33 promotion of logging and narrowing of data_access.

- **`CHANGELOG.md`** — Added `Changed` section entry under `[Unreleased]` documenting the per-category distribution shift, the escape hatch, and the M20 epsilon note.

- **`MIGRATION_NOTES.md`** — Added M33 section with worked example (pre/post `pattern_metrics` JSON from `simple-typescript` fixture), escape hatch TOML snippet, and foreign-extractor guidance.

## Root Cause (bugs only)
N/A — feature milestone

## Files Modified
- `crates/sdivi-patterns/src/catalog.rs`
- `crates/sdivi-patterns/src/queries/mod.rs`
- `crates/sdivi-patterns/tests/m33_sentinels.rs` (NEW)
- `crates/sdivi-patterns/tests/logging_fixture.rs`
- `crates/sdivi-pipeline/tests/snapshot_m32_unchanged.rs`
- `tests/fixtures/simple-typescript/app.ts`
- `tests/fixtures/simple-rust/src/lib.rs`
- `tests/fixtures/simple-rust/src/utils.rs`
- `tests/fixtures/simple-python/main.py`
- `docs/pattern-categories.md`
- `CHANGELOG.md`
- `MIGRATION_NOTES.md`

## Human Notes Status
No human notes provided in this task.

## Docs Updated
- `docs/pattern-categories.md` — per-language tables, canonical category list, embedder responsibilities, callee-text classification header.
- `CHANGELOG.md` — Changed section under [Unreleased].
- `MIGRATION_NOTES.md` — M33 section with worked example.

## Observed Issues (out of scope)
- `wasm_package_json_version_matches_workspace` — pre-existing: package.json at 0.2.18 vs workspace 0.2.22. Not introduced by M33.
- `data_access_fixture.rs::call_expression_maps_to_data_access_for_go` — conceptually misleading post-M33 (tests `category_for_node_kind`, which still maps all `call_expression` to data_access, but the pipeline now uses `classify_hint` which filters); test is still correct for what it asserts but no longer documents actual pipeline behavior.
