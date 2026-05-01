# Coder Summary — M15: Change-Coupling Analyzer
_Branch: milestones/v0 | Cycle: M15_

## Status: COMPLETE

## What Was Implemented

Implemented the full M15 change-coupling analyzer across all layers:

1. **Weighted Leiden support** (`sdi-detection`): `LeidenGraph` now carries `edge_weights: Vec<Vec<f64>>` parallel to `adj`. New `from_edges_weighted`, `from_dependency_graph_weighted` constructors. `ModularityState`, `compute_stability`, `compute_modularity`, `aggregate_network` all updated to use weighted edge sums. New public `run_leiden_with_weights` function.

2. **Change-coupling snapshot types** (`sdi-snapshot`): New `change_coupling.rs` with `CoChangePair` and `ChangeCouplingResult`. `Snapshot` struct gains `change_coupling: Option<ChangeCouplingResult>` field. `assemble_snapshot` takes an additional `change_coupling` parameter.

3. **Pure-compute API** (`sdi-core`): New input types `CoChangeEventInput` and `ChangeCouplingConfigInput` in `input/change_coupling_types.rs`. `LeidenConfigInput` gains `edge_weights: Option<BTreeMap<(String, String), f64>>`. New `compute/change_coupling.rs` with `compute_change_coupling` pure function. `AnalysisError` gains `InvalidConfig` variant. `detect_boundaries` uses weighted Leiden when `cfg.edge_weights` is set. All new items re-exported from `sdi-core`.

4. **Pipeline I/O** (`sdi-pipeline`): New `change_coupling.rs` with `collect_cochange_events` (git shell-out, NUL-separated output parsing). `pipeline.rs` integrates change-coupling collection before Stage 3, passes `change_coupling_result` to `assemble_snapshot`, builds `weight_map` for weighted Leiden when `boundaries.weighted_edges = true`. New `build_edge_weight_map` helper.

5. **CLI output** (`sdi-cli`): `print_snapshot` in `output/text.rs` now shows change-coupling pair count and top-5 pairs.

6. **WASM bindings** (`sdi-wasm`): New `change_coupling.rs` with `WasmCoChangeEventInput`, `WasmChangeCouplingConfigInput`, `WasmCoChangePair`, `WasmChangeCouplingResult`, and `compute_change_coupling` WASM export. `SNAPSHOT_TS` updated with `ChangeCouplingResult` and `CoChangePair` TypeScript interfaces.

7. **Tests**: 9-case unit test for `compute_change_coupling`; 4-case git integration test for `collect_cochange_events`; 2-case lifecycle test; 1-case weighted Leiden test.

## Root Cause (bugs only)
N/A — feature implementation

## Files Modified

**New files:**
- `crates/sdi-snapshot/src/change_coupling.rs`
- `crates/sdi-core/src/input/change_coupling_types.rs`
- `crates/sdi-core/src/compute/change_coupling.rs`
- `crates/sdi-pipeline/src/change_coupling.rs`
- `bindings/sdi-wasm/src/change_coupling.rs`
- `crates/sdi-core/tests/compute_change_coupling.rs`
- `crates/sdi-pipeline/tests/change_coupling_git.rs`
- `crates/sdi-pipeline/tests/change_coupling_lifecycle.rs`
- `crates/sdi-detection/tests/leiden_weighted_edges.rs`
- `tests/change_coupling_lifecycle.rs` (placeholder — actual test in sdi-pipeline)

**Modified files:**
- `crates/sdi-core/src/error.rs` — added `InvalidConfig` variant
- `crates/sdi-detection/src/leiden/graph.rs` — full rewrite with weighted support
- `crates/sdi-detection/src/leiden/modularity.rs` — full rewrite with weighted sums
- `crates/sdi-detection/src/leiden/quality.rs` — full rewrite with weighted edge counts
- `crates/sdi-detection/src/leiden/aggregate.rs` — full rewrite with weighted edge accumulation
- `crates/sdi-detection/src/leiden/mod.rs` — fixed weighted k_in, added `run_leiden_with_weights`
- `crates/sdi-detection/src/lib.rs` — added `pub use leiden::run_leiden_with_weights`
- `crates/sdi-snapshot/src/lib.rs` — added `change_coupling` module + re-exports
- `crates/sdi-snapshot/src/snapshot.rs` — added `change_coupling` field, updated signature
- `crates/sdi-snapshot/src/delta.rs` — updated `assemble_snapshot` calls
- `crates/sdi-snapshot/src/trend.rs` — updated `assemble_snapshot` call
- `crates/sdi-snapshot/tests/boundary_spec_assembly.rs` — updated calls
- `crates/sdi-snapshot/tests/delta_pure.rs` — updated call
- `crates/sdi-snapshot/tests/compute_trend.rs` — updated call
- `crates/sdi-snapshot/tests/null_vs_zero.rs` — updated call
- `crates/sdi-snapshot/tests/atomic_write.rs` — updated call
- `crates/sdi-snapshot/tests/snapshot_load.rs` — updated call
- `crates/sdi-snapshot/tests/retention.rs` — updated call
- `crates/sdi-core/src/input/mod.rs` — added `change_coupling_types` module
- `crates/sdi-core/src/input/types.rs` — added `edge_weights` to `LeidenConfigInput`
- `crates/sdi-core/src/compute/mod.rs` — added `change_coupling` module
- `crates/sdi-core/src/compute/boundaries.rs` — weighted Leiden in `detect_boundaries`
- `crates/sdi-core/src/lib.rs` — added new re-exports
- `crates/sdi-pipeline/src/lib.rs` — added `change_coupling` module + re-exports
- `crates/sdi-pipeline/src/pipeline.rs` — integrated change-coupling analysis + weighted Leiden
- `crates/sdi-pipeline/src/boundaries.rs` — updated `assemble_snapshot` call
- `crates/sdi-cli/src/output/text.rs` — added change-coupling display in `print_snapshot`
- `bindings/sdi-wasm/src/lib.rs` — added `change_coupling` module + TypeScript interfaces
- `bindings/sdi-wasm/src/exports.rs` — updated `assemble_snapshot` call
- `CHANGELOG.md` — added M15 entry
- `docs/library-embedding.md` — added change-coupling section
- `docs/cli-integration.md` — added weighted community detection section
- `docs/determinism.md` — added change-coupling determinism section

## Human Notes Status
N/A

## Docs Updated
- `CHANGELOG.md` — M15 entry added under `[Unreleased]`
- `docs/library-embedding.md` — "Computing change-coupling from a foreign extractor" section
- `docs/cli-integration.md` — "Change-coupling and weighted community detection" section
- `docs/determinism.md` — "Change-coupling determinism" section

## Architecture Change Proposals
None — implementation follows the M15 spec exactly.

## Observed Issues (out of scope)
- Pre-existing test failure: `workspace_version_is_0_1_0` in `sdi-cli` checks for version "0.1.0" but workspace is at "0.1.1". This test was failing before M15.
- `git --no-pager` must be a git flag (before subcommand) not a `log` flag — fixed during implementation.

## Files Modified (auto-detected)
- `.claude/milestones/MANIFEST.cfg`
- `.claude/milestones/m15-change-coupling-analyzer.md`
- `.tekhton/CODER_SUMMARY.md`
- `.tekhton/DRIFT_LOG.md`
- `.tekhton/HUMAN_ACTION_REQUIRED.md`
- `.tekhton/PREFLIGHT_REPORT.md`
- `.tekhton/REVIEWER_REPORT.md`
- `.tekhton/TESTER_REPORT.md`
- `.tekhton/test_dedup.fingerprint`
- `CHANGELOG.md`
- `Cargo.lock`
- `bindings/sdi-wasm/src/exports.rs`
- `bindings/sdi-wasm/src/lib.rs`
- `bindings/sdi-wasm/src/types.rs`
- `crates/sdi-cli/src/output/text.rs`
- `crates/sdi-core/src/compute/boundaries.rs`
- `crates/sdi-core/src/compute/mod.rs`
- `crates/sdi-core/src/compute/patterns.rs`
- `crates/sdi-core/src/error.rs`
- `crates/sdi-core/src/input/mod.rs`
- `crates/sdi-core/src/input/types.rs`
- `crates/sdi-core/src/lib.rs`
- `crates/sdi-core/tests/compute_thresholds_check.rs`
- `crates/sdi-detection/src/leiden/aggregate.rs`
- `crates/sdi-detection/src/leiden/graph.rs`
- `crates/sdi-detection/src/leiden/mod.rs`
- `crates/sdi-detection/src/leiden/modularity.rs`
- `crates/sdi-detection/src/leiden/quality.rs`
- `crates/sdi-detection/src/lib.rs`
- `crates/sdi-patterns/src/fingerprint.rs`
