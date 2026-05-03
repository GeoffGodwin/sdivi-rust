# Coder Summary
## Status: COMPLETE

## What Was Implemented
- Added `pub const THRESHOLD_EPSILON: f64 = 1e-9` to `crates/sdivi-core/src/compute/threshold_types.rs` (new file, extracted from `thresholds.rs`) with a doc comment citing `docs/determinism.md § Threshold gate stability`.
- Replaced all six `delta > limit` comparisons in `compute_thresholds_check` with `delta > limit + THRESHOLD_EPSILON` (four aggregate dimensions + two per-category loops). The `boundary_violation_delta` integer cast comment explains epsilon has no functional effect there.
- Re-exported `THRESHOLD_EPSILON` from `sdivi-core::lib` alongside the other threshold types so WASM callers and other embedders can reference the constant.
- Added "Threshold gate stability" subsection to `docs/determinism.md` explaining the constant value, the asymmetric application (epsilon is added to limit, not subtracted from delta), and the guarantee that genuine breaches above `limit + 2e-9` still trip.
- Updated `CHANGELOG.md` under `[0.1.10]` with the behaviour-change note.
- Added unit tests in `crates/sdivi-core/tests/thresholds_epsilon.rs` (NEW): two cases per dimension (`limit + 5e-10` does not breach; `limit + 2e-9` does breach), plus per-category path tests and THRESHOLD_EPSILON regression gate.
- Added property test `prop_breach_equals_delta_gt_limit_plus_epsilon` to `prop_thresholds.rs` asserting `breached(delta, limit) == (delta > limit + THRESHOLD_EPSILON)` for any limit and delta.

## Root Cause (bugs only)
N/A — this is a feature milestone.

## Files Modified
- `crates/sdivi-core/src/compute/threshold_types.rs` (NEW) — public types (THRESHOLD_EPSILON, ThresholdBreachInfo, AppliedOverrideInfo, ThresholdCheckResult) extracted from thresholds.rs to keep thresholds.rs under 300 lines
- `crates/sdivi-core/src/compute/thresholds.rs` — replaced 6 `delta > limit` comparisons with `delta > limit + THRESHOLD_EPSILON`; added `pub use super::threshold_types::*`
- `crates/sdivi-core/src/compute/mod.rs` — added `mod threshold_types`
- `crates/sdivi-core/src/lib.rs` — added `THRESHOLD_EPSILON` to the compute::thresholds re-export
- `crates/sdivi-core/tests/thresholds_epsilon.rs` (NEW) — M20 epsilon boundary unit tests
- `crates/sdivi-core/tests/thresholds_overrides.rs` (NEW) — M14 per-category override tests (extracted from compute_thresholds_check.rs to keep it under 300 lines)
- `crates/sdivi-core/tests/compute_thresholds_check.rs` — removed M14/drift override tests (moved to thresholds_overrides.rs); removed unused `cfg_with_drift_override` helper
- `crates/sdivi-core/tests/prop_thresholds.rs` — added `prop_breach_equals_delta_gt_limit_plus_epsilon` property test
- `docs/determinism.md` — added "Threshold gate stability" subsection
- `CHANGELOG.md` — added `[0.1.10]` entry under Changed

## Docs Updated
- `docs/determinism.md` — "Threshold gate stability" subsection added
- `CHANGELOG.md` — entry for M20 behaviour change

## Human Notes Status
N/A — no Human Notes section in this task.

## Observed Issues (out of scope)
- `crates/sdivi-patterns/src/catalog.rs:7` — unused import `sdivi_config::PatternsConfig` (pre-existing)
- `crates/sdivi-patterns/src/catalog.rs:10` — unused import `fingerprint_node_kind` (pre-existing)
- `crates/sdivi-patterns/src/catalog.rs:11` — unused import `crate::queries` (pre-existing)
- `crates/sdivi-graph/src/dependency_graph.rs` — one unused import warning (pre-existing, `cargo fix` suggestion)
