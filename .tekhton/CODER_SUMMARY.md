# Coder Summary
## Status: COMPLETE

## What Was Implemented
All 10 non-blocking tech debt items from `.tekhton/NON_BLOCKING_LOG.md` addressed:

1. **`commit_extract.rs` — renamed error variant** Added `CommitDateParseFailed { sha, raw }` to `CommitExtractError`; `commit_date_iso` now returns this variant instead of `RefResolutionFailed` when the date string cannot be parsed.
2. **`docs/library-embedding.md` audit gap** The M16 CODER_SUMMARY.md listed `docs/library-embedding.md` in "Docs Updated" but omitted it from "Files Modified". Confirmed the file was genuinely updated in M16 (git log). In this run, added a new "Weighted Leiden — `edge_weight_key`" section to document the public API added by item 9.
3. **`commit_extract.rs:94-113` — tar check before git archive** The `tar --version` availability check now precedes the `git archive` spawn so we don't abandon a running process.
4. **Security findings** MEDIUM/LOW findings (rev-parse `--` separator, tar `--no-absolute-filenames`, stderr truncation) handled by the security pipeline. No code change required.
5. **`change_coupling.rs:96-100` — dead else branch removed** `pair_counts` key now written directly as `(sorted_files[i].clone(), sorted_files[j].clone())` since `i < j` on sorted input guarantees the canonical order.
6. **`change_coupling.rs:101-103,123` — double empty-SHA guard removed** The inner `if !sha.is_empty()` guard at the point of constructing `CoChangeEventInput` was removed; the outer `if sha.is_empty() { continue }` at the top of the block is the single guard.
7. **`exports.rs:168` — `change_coupling: None` limitation documented** Added a TODO comment in `assemble_snapshot` explaining that WASM consumers calling `compute_change_coupling` cannot include the result in the assembled snapshot without a follow-up field addition.
8. **`types.rs:53-58` — missing `edge_weights` field documented** Updated `WasmLeidenConfigInput` doc comment to explicitly state that WASM Leiden is always unweighted and `edge_weights` is absent.
9. **`input/types.rs:145` — `serde_json` tuple-key serialization fixed** Changed `LeidenConfigInput::edge_weights` from `Option<BTreeMap<(String, String), f64>>` to `Option<BTreeMap<String, f64>>` with NUL-delimited `"source\x00target"` string keys. Added `edge_weight_key(source, target) -> String` and `split_edge_weight_key(key) -> Option<(&str, &str)>` helpers in a new `crates/sdi-core/src/input/edge_weight.rs` module. Both helpers are re-exported from `sdi_core::input` and `sdi_core`. A `leiden_config_serde.rs` integration test verifies round-trip serialization with populated `edge_weights`.
10. **`change_coupling.rs:83` — `HashSet` → `BTreeSet`** `all_files: BTreeSet<String>` aligns with the project's `BTreeSet` convention for ordered collections.

## Root Cause (bugs only)
N/A — all items are tech debt cleanup, not bugs.

## Files Modified
- `crates/sdi-pipeline/src/commit_extract.rs` — items 1, 3: new error variant + tar-before-spawn ordering
- `crates/sdi-core/src/compute/change_coupling.rs` — items 5, 10: remove dead else branch, HashSet→BTreeSet
- `crates/sdi-pipeline/src/change_coupling.rs` — item 6: remove double empty-SHA guard
- `bindings/sdi-wasm/src/exports.rs` — item 7: TODO comment on hardcoded `change_coupling: None`
- `bindings/sdi-wasm/src/types.rs` — item 8: doc comment on absent `edge_weights` field
- `crates/sdi-core/src/input/types.rs` — item 9: changed `edge_weights` key type to `String`
- `crates/sdi-core/src/input/mod.rs` — item 9: re-export `edge_weight_key`, `split_edge_weight_key`
- `crates/sdi-core/src/input/edge_weight.rs` (NEW) — item 9: `edge_weight_key` + `split_edge_weight_key` helpers
- `crates/sdi-core/src/lib.rs` — item 9: re-export `edge_weight_key`, `split_edge_weight_key` from crate root
- `crates/sdi-core/tests/leiden_config_serde.rs` — item 9: integration tests for edge_weights serde round-trip
- `crates/sdi-core/src/compute/boundaries.rs` — incidental; no logic change
- `docs/library-embedding.md` — item 2/9: added "Weighted Leiden — `edge_weight_key`" section

## Docs Updated
- `docs/library-embedding.md` — added "Weighted Leiden — `edge_weight_key`" section documenting the new public API (`edge_weight_key`, `split_edge_weight_key`).

## Human Notes Status
1. [COMPLETED] `commit_extract.rs:81` — `CommitDateParseFailed { sha, raw }` variant added.
2. [COMPLETED] `CODER_SUMMARY.md` audit gap — confirmed file was updated in M16; this run adds `edge_weight_key` docs to the file and properly lists it in Files Modified.
3. [COMPLETED] `commit_extract.rs:94-113` — tar availability check moved before `git archive` spawn.
4. [COMPLETED] Security findings (MEDIUM/LOW) — no code change required; noted for completeness as instructed.
5. [COMPLETED] `change_coupling.rs:96-100` — dead else branch removed; direct pair key written.
6. [COMPLETED] `change_coupling.rs:101-103,123` — double empty-SHA guard removed.
7. [COMPLETED] `exports.rs:168` — TODO comment added documenting `change_coupling: None` limitation.
8. [COMPLETED] `types.rs:53-58` — doc comment added noting `edge_weights` is absent from WASM Leiden.
9. [COMPLETED] `input/types.rs:145` — `edge_weights` uses `BTreeMap<String, f64>` with NUL-sep keys; `edge_weight_key`/`split_edge_weight_key` helpers added; serde round-trip tested.
10. [COMPLETED] `change_coupling.rs:83` — `HashSet<String>` changed to `BTreeSet<String>`.

## Observed Issues (out of scope)
- `crates/sdi-config/src/thresholds.rs:46` — `validate_and_prune_overrides` is pub(crate) but never used (compiler warning).
- `crates/sdi-graph/src/dependency_graph.rs:9` — unused import `tracing::debug` (compiler warning).
- `crates/sdi-patterns/src/catalog.rs` — several unused imports and dead functions (`build_globset`, `is_excluded`) (compiler warnings).

## Files Modified (auto-detected)
- `.tekhton/CODER_SUMMARY.md`
- `.tekhton/PREFLIGHT_REPORT.md`
- `.tekhton/REVIEWER_REPORT.md`
- `.tekhton/TESTER_REPORT.md`
- `Cargo.lock`
- `bindings/sdi-wasm/src/exports.rs`
- `bindings/sdi-wasm/src/types.rs`
- `crates/sdi-core/src/compute/boundaries.rs`
- `crates/sdi-core/src/compute/change_coupling.rs`
- `crates/sdi-core/src/input/mod.rs`
- `crates/sdi-core/src/input/types.rs`
- `crates/sdi-core/src/lib.rs`
- `crates/sdi-core/tests/leiden_config_serde.rs`
- `crates/sdi-pipeline/src/change_coupling.rs`
- `crates/sdi-pipeline/src/commit_extract.rs`
- `docs/library-embedding.md`
