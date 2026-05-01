# Coder Summary
## Status: COMPLETE

## What Was Implemented
- Added `convention_drift_per_category: BTreeMap<String, f64>` to `PatternMetricsResult`
- Added `pattern_entropy_per_category_delta` / `convention_drift_per_category_delta` to `DivergenceSummary`
- Updated `null_summary()` to return `None` for both new per-category delta fields
- Updated `compute_delta` to populate both per-category delta maps using union-of-keys with zero-fill
- Implemented override expiry logic in `compute_thresholds_check` — active overrides raise per-category limits; expired overrides fall back to global rate; aggregate dimensions always use global rate
- Added `category: Option<String>` to `ThresholdBreachInfo` — `None` for aggregate breaches, `Some(cat)` for per-category
- Added `applied_overrides: BTreeMap<String, AppliedOverrideInfo>` to `ThresholdCheckResult` — diagnostic surface
- Added `AppliedOverrideInfo { active, expires, expired_reason }` struct re-exported from `sdi-core::lib`
- Extracted time utilities to `crates/sdi-pipeline/src/time.rs`
- Updated pipeline's `compute_pattern_metrics_from_catalog` to populate `convention_drift_per_category`
- Retired all `TODO(M09)`, `M08`/`M09`-named stub tests
- Renamed `override_not_wired_in_m08_base_rate_applies` → `active_override_raises_per_category_limit`
- Renamed `base_rate_applies_regardless_of_override_state_m08` → `expired_override_falls_back_to_global_rate`
- Added new unit tests: `active_override_blocks_per_category_breach`, `aggregate_dimension_uses_global_rate_when_only_one_category_is_overridden`, `applied_overrides_reports_active_and_expired_separately`, `expiry_boundary_today_equals_expires_is_still_active`, `category_present_in_curr_only_surfaces_positive_delta`, `category_present_in_prev_only_surfaces_negative_delta`
- Extended prop tests with `prop_thresholds_with_overrides_pure` and `prop_per_category_delta_pure`
- Updated CLI text output to show applied overrides; updated JSON output to pass through `applied_overrides`
- Updated WASM types with `WasmAppliedOverrideInfo`, `applied_overrides` on `WasmThresholdCheckResult`, `category` on `WasmThresholdBreachInfo`, `convention_drift_per_category` on `WasmPatternMetricsResult`
- Added `bindings/sdi-wasm/src/assemble_types.rs` (extracted from `types.rs`)
- Added `crates/sdi-pipeline/tests/parity.rs` — parity test for `convention_drift_per_category`
- Added `crates/sdi-cli/tests/check_format.rs` — `applied_overrides` round-trip test
- Updated `CHANGELOG.md` with M14 entry
- Updated `docs/library-embedding.md` with Meridian-style override example
- Fixed wasm smoke test struct literal to include `convention_drift_per_category` field

## Root Cause (bugs only)
(not a bug-fix task)

## Files Modified
- `crates/sdi-snapshot/src/snapshot.rs` — add `convention_drift_per_category` to PatternMetricsResult
- `crates/sdi-snapshot/src/delta.rs` — add two new per-category delta fields to DivergenceSummary
- `crates/sdi-snapshot/tests/null_vs_zero.rs` — add assertions for new null fields
- `crates/sdi-core/src/compute/patterns.rs` — populate `convention_drift_per_category`
- `crates/sdi-pipeline/src/time.rs` (NEW) — extract current_timestamp + unix_to_iso8601
- `crates/sdi-pipeline/src/pipeline.rs` — remove time utils, add convention_drift_per_category
- `crates/sdi-pipeline/src/lib.rs` — update re-exports
- `crates/sdi-core/src/compute/thresholds.rs` — implement override wiring, delete TODO(M09)
- `crates/sdi-core/src/lib.rs` — re-export AppliedOverrideInfo
- `crates/sdi-core/tests/compute_thresholds_check.rs` — rename+rewrite stubs, add 6 new tests
- `crates/sdi-core/tests/prop_thresholds.rs` — extend with overrides and per-category prop tests
- `bindings/sdi-wasm/src/assemble_types.rs` (NEW) — WasmAssembleSnapshotInput extracted from types.rs
- `bindings/sdi-wasm/src/types.rs` — add new fields and WasmAppliedOverrideInfo
- `bindings/sdi-wasm/src/lib.rs` — add assemble_types mod
- `bindings/sdi-wasm/tests/wasm_smoke.rs` — add missing `convention_drift_per_category` field; trim to 300 lines
- `crates/sdi-cli/src/output/json.rs` — use actual applied_overrides
- `crates/sdi-cli/src/output/text.rs` — show applied overrides
- `crates/sdi-pipeline/tests/parity.rs` (NEW) — convention_drift_per_category parity test
- `crates/sdi-cli/tests/check_format.rs` (NEW) — applied_overrides JSON round-trip test
- `CHANGELOG.md` — M14 entry
- `docs/library-embedding.md` — Meridian-style override example

## Docs Updated
- `docs/library-embedding.md` — added Meridian-style caller example showing `today` and `overrides`
- `crates/sdi-core/src/compute/thresholds.rs` — updated rustdoc on `compute_thresholds_check`, `ThresholdBreachInfo`, `ThresholdCheckResult`, `AppliedOverrideInfo`
- `crates/sdi-snapshot/src/snapshot.rs` — rustdoc on new `convention_drift_per_category` field
- `crates/sdi-snapshot/src/delta.rs` — rustdoc on two new per-category delta fields

## Human Notes Status
(no Human Notes section in milestone)

## Observed Issues (out of scope)
- `.claude/milestones/MANIFEST.cfg` and `m14-per-category-threshold-override-wiring.md` — could not update `status: done` because these files are protected. Requires manual update or elevated permissions.
- `crates/sdi-config/src/thresholds.rs:46` — `validate_and_prune_overrides` is unused (pre-existing warning)
- `crates/sdi-graph/src/dependency_graph.rs:9` — unused import `tracing::debug` (pre-existing warning)
- `crates/sdi-patterns/src/catalog.rs` — multiple pre-existing unused imports and dead code warnings
