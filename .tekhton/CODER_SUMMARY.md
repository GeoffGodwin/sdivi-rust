# Coder Summary
## Status: COMPLETE

## What Was Implemented

### From Prior Runs (1 and 2 — preserved context)
- Workspace Cargo.toml: added `sdi-pipeline` member/dep, `chrono` workspace dep
- `sdi-config`: added `loader` feature gate; `validate_overrides_format` (pure, no clock); `today_iso8601()` (clock, loader-gated); override pruning retained in `validate_and_prune_overrides`
- `sdi-graph`: added `pipeline-records` feature; `build_dependency_graph_from_edges` constructor
- `sdi-detection`: added `pipeline-records` feature; removed `tempfile` from runtime deps; `warm_start.rs` retains pure mapping logic only (FS moved to sdi-pipeline::cache); Leiden community-ID offset fix (subtraction overflow bug from M05)
- `sdi-patterns`: added `pipeline-records` feature; `normalize.rs` with `normalize_and_hash`; `fingerprint_node_kind` updated to thin wrapper
- `sdi-snapshot`: `pipeline-records` feature; renamed `build_snapshot`→`assemble_snapshot`; added `PatternMetricsResult`, `convention_drift_delta`, `trend.rs`, `boundary_inference.rs`
- `sdi-pipeline` (NEW crate): `Pipeline` orchestration with `cache.rs`, `store.rs`
- `sdi-core`: reshaped to pure-compute facade with `input/`, `compute/` modules, `facade.rs`, `error.rs`
- `sdi-cli`: updated to use `sdi_pipeline::Pipeline`
- `CHANGELOG.md`: M08 entries
- `sdi-detection/tests/leiden_id_collision.rs` (tester)
- `sdi-config/src/thresholds.rs` boundary tests (tester)

### This Run (attempt 3 — completion pass)
- Fixed `sdi-cli/tests/version.rs` version string 0.0.8→0.0.9
- Split `sdi-core/src/input.rs` (373 lines) into `input/mod.rs` (84 lines) + `input/types.rs` (290 lines) to satisfy the 300-line ceiling
- Deleted dead `sdi-core/src/pipeline.rs` (M07 leftover not exported from lib.rs)
- Created all 10 missing test files from the milestone spec:
  - `crates/sdi-core/tests/validate_node_id.rs` — 11 tests covering all rejection cases
  - `crates/sdi-core/tests/normalize_and_hash.rs` — M07 equivalence regression + property tests
  - `crates/sdi-core/tests/compute_topology.rs` — coupling topology across graph topologies
  - `crates/sdi-core/tests/compute_pattern_metrics.rs` — entropy, convention_drift formula
  - `crates/sdi-core/tests/compute_thresholds_check.rs` — null-summary path, breach detection, override expiry
  - `crates/sdi-core/tests/leiden_disconnected.rs` — disconnected component detection
  - `crates/sdi-core/tests/leiden_historical_stability.rs` — stability scoring with various prior histories
  - `crates/sdi-core/tests/wasm_compat.rs` — smoke tests for all compute functions (verifies no I/O deps needed)
  - `crates/sdi-snapshot/tests/compute_trend.rs` — trend slopes, last_n clamping, empty/single inputs
  - `crates/sdi-snapshot/tests/infer_boundaries.rs` — stability gating, community-ID renaming, ordering
- Created `crates/sdi-pipeline/tests/pipeline_smoke.rs` — Pipeline::new, snapshot, and delta smoke tests on simple-rust fixture
- Updated `CHANGELOG.md` with detailed M08 change list

## Root Cause (bugs only)
1. `version.rs` test: crate bumped to 0.0.9 but test still checked 0.0.8
2. `input.rs` at 373 lines exceeded the 300-line hard ceiling — split into submodule
3. `leiden_historical_stability.rs` initial test: `community_sets_match` compares node-set membership not numeric IDs — singleton communities with swapped IDs still match. Fixed test to use non-singleton communities with actual membership changes.

## Files Modified
- `crates/sdi-cli/tests/version.rs` — version string 0.0.8→0.0.9
- `crates/sdi-core/src/input.rs` → DELETED (replaced by submodule)
- `crates/sdi-core/src/input/mod.rs` (NEW) — validate_node_id + re-exports (84 lines)
- `crates/sdi-core/src/input/types.rs` (NEW) — all input struct definitions (290 lines)
- `crates/sdi-core/src/pipeline.rs` — DELETED (dead code, not exported from lib.rs)
- `crates/sdi-core/tests/validate_node_id.rs` (NEW)
- `crates/sdi-core/tests/normalize_and_hash.rs` (NEW)
- `crates/sdi-core/tests/compute_topology.rs` (NEW)
- `crates/sdi-core/tests/compute_pattern_metrics.rs` (NEW)
- `crates/sdi-core/tests/compute_thresholds_check.rs` (NEW)
- `crates/sdi-core/tests/leiden_disconnected.rs` (NEW)
- `crates/sdi-core/tests/leiden_historical_stability.rs` (NEW)
- `crates/sdi-core/tests/wasm_compat.rs` (NEW)
- `crates/sdi-snapshot/tests/compute_trend.rs` (NEW)
- `crates/sdi-snapshot/tests/infer_boundaries.rs` (NEW)
- `crates/sdi-pipeline/tests/pipeline_smoke.rs` (NEW)
- `CHANGELOG.md` — detailed M08 Added/Changed entries

## Docs Updated
CHANGELOG.md — comprehensive M08 entries covering: sdi-pipeline new crate, sdi-core reshape, new compute modules, assemble_snapshot rename, Snapshot.pattern_metrics addition, DivergenceSummary.convention_drift_delta addition, normalize_and_hash extension, override-expiry single-source move.

## Human Notes Status
No Human Notes present in this task.

## Observed Issues (out of scope)
- `sdi-detection/src/leiden/mod.rs:168` — comment overstates invariant (noted in reviewer as non-blocking; not addressed per scope rules)
- `sdi-config/src/thresholds.rs:46` — `validate_and_prune_overrides` shows dead_code warning; it IS called from `load.rs` but the compiler may not see it due to feature-gating. Out of scope to investigate further.
- `sdi-patterns/src/catalog.rs` — multiple unused import warnings (`PatternsConfig`, `fingerprint_node_kind`, `crate::queries`). Pre-existing; out of scope.
- `sdi-graph/src/dependency_graph.rs:9` — unused `tracing::debug` import. Pre-existing; out of scope.
- MANIFEST.cfg and m08 milestone file could not be updated to `done` status (permission denied on sensitive files). Pipeline/human should mark M08 done.
