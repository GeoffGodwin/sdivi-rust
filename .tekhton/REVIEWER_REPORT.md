# Reviewer Report — M33: Native Pipeline Switchover to `classify_hint`
Review cycle: 1 of 4
Reviewed by: reviewer agent

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `queries/mod.rs:24-31` — The `ALL_CATEGORIES` const doc comment still reads "Note: `logging` is a catalog-only category for `snapshot_version "1.0"`" and implies the native pipeline never populates it. M33 makes this framing stale — logging is now natively classified via `classify_hint`. The individual claim that "`category_for_node_kind` never returns `Some("logging")`" remains accurate (the M30 sentinel confirms it), but the "catalog-only" label should be updated to say logging is natively classified since M33.
- `prop_classify_hint.rs` appears as modified in gitStatus but is absent from CODER_SUMMARY "Files Modified". The file content is correct and valuable — it tests the fall-through consistency invariant for non-special node kinds. The omission is a CODER_SUMMARY accuracy gap only; nothing is wrong with the code.
- `MIGRATION_NOTES.md` worked example shows pre-M33 `data_access.instance_count: 2`. Milestone spec explicitly requires this be generated from a real fixture run, not estimated. If hand-authored, confirm against an actual pipeline run before tagging the release — the spec warns "an inaccurate example is worse than no example."

## Coverage Gaps
- No integration test covers the `simple-go` fixture acceptance criterion: "data_access containing only `db.*`/`sql.*`-shape calls and `logging` containing only `fmt.Print*`-shape calls; non-matching calls dropped from both." This is an explicit M33 acceptance criterion not covered by any modified test file.
- WASM re-baselining: CODER_SUMMARY does not confirm `bindings/sdivi-wasm` integration tests were checked. If no WASM test asserts pattern_metrics distribution shape the criterion is vacuously satisfied — PR description should note this explicitly so reviewers don't assume a check was done.

## Drift Observations
- `crates/sdivi-pipeline/tests/snapshot_m32_unchanged.rs` — The file name and determinism test (`m32_pipeline_output_byte_identical_for_same_params`) are labelled M32 but the file now hosts the M33 positive sentinel (`m33_pipeline_snapshot_has_logging_entry_for_tracing_macros`). The M32 determinism test is still correct and load-bearing. Consider renaming the file to `snapshot_pipeline_regression.rs` in a future cleanup pass to avoid reader confusion.
- `catalog.rs:110-113` — `crate::hint_input::PatternHintInput` is referenced by its full path inline rather than imported at the top of the file via `use`. The other feature-gated items (`fingerprint_node_kind`, `queries`) are imported normally. Cosmetic inconsistency only.
- `resource_management::excludes_callee` and `logging::matches_callee` are both tested in the `macro_invocation` arm of `classify_hint` — they use the same regex in v0, so the double check is harmless but redundant. The prior M32 Drift Observation about these two identical `RUST_LOGGING_RE` / `RUST_RE` literals being maintained separately remains open; M33 does not worsen it.
