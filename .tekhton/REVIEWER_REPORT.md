# Reviewer Report — Milestone 11: Documentation, Examples, Determinism Polish, bifl-tracker Validation
Review cycle: 1 of 4

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `docs/determinism.md` says `compute_thresholds_check` "takes `today: NaiveDate` as a parameter" but `today` is embedded inside `ThresholdsInput`, not a standalone argument — mildly misleading for WASM callers reading this doc.
- `tools/validate-against-bifl-tracker.sh:27` — variable is named `SDE_BIN` (typo); should be `SDI_BIN`. Works correctly since it is used consistently, but confusing.
- `docs/determinism.md` property tests table is missing two new tests added in this PR: `prop_test_pipeline_deterministic` (sdi-pipeline) and `prop_none_delta_never_breaches` (sdi-core).
- `crates/sdi-cli/examples/embed_compute.rs:108-114` — parity check compares `reference.graph.node_count` against `coupling.node_count` built from `path_partition.keys()`. Isolated nodes (no community assignment) appear in the graph count but not in `path_partition`, so the check can print `match=false` on a correct implementation. Comment should warn this is an approximate check, not an identity assertion.

## Coverage Gaps
- No test covers the `.sdi/`-exclusion branch of the `NoGrammarsAvailable` check: a repo where `records.is_empty()` but all extensioned files are inside `.sdi/` (e.g., only snapshot JSON) should NOT trigger exit 3. The current test only exercises the positive case (extensioned file outside `.sdi/`).

## Drift Observations
- `crates/sdi-pipeline/src/pipeline.rs:160-163` — `save_cached_partition` executes unconditionally regardless of `WriteMode`. `EphemeralForCheck` suppresses the snapshot write but still mutates `tests/fixtures/simple-rust/.sdi/cache/partition.json` when `prop_test_pipeline_deterministic` runs. This makes a checked-in fixture a mutable side-effect of the new prop test. Pre-existing pipeline behavior, but the new prop test amplifies the cross-run state mutation.
- `tools/validate-against-bifl-tracker.sh:119` — commit SHA is extracted via a Python one-liner with bash-interpolated `$baseline_file`. If `BASELINES_DIR` ever contains a single-quote in a path component the Python command breaks. Low risk for a tool-controlled path, but heredoc or temp-file approach would be more robust.
- `crates/sdi-cli/examples/embed_compute.rs:63-83` — the "pure-compute path" example re-invokes `sdi_parsing::parse_repository` to extract edges, introducing a FS dependency. Comment "In a real consumer app the caller supplies these" correctly flags this, but the example header says "pure-compute path (WASM-path)" and may mislead readers who expect it to be runnable without FS/tree-sitter.
