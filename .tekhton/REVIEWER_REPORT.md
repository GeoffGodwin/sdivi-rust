# Reviewer Report — M20: Threshold-Comparison Epsilon for Cross-Arch Stability

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `threshold_types.rs:98` — the `ThresholdCheckResult` doc-test imports `sdivi_snapshot::delta::null_summary` directly rather than the re-exported `sdivi_core::null_summary`. Doc-test examples in `sdivi-core` should use the public re-export path that consumers will see on docs.rs, not the internal crate path.

## Coverage Gaps
- `prop_breach_equals_delta_gt_limit_plus_epsilon` only exercises the `pattern_entropy_delta` dimension. The milestone's property-test spec reads as dimension-agnostic ("for any `limit > 0` and any `delta`"). Adding three more cases — one each for `convention_drift_delta`, `coupling_delta`, and `boundary_violation_delta` — would guard against a future per-dimension refactor that accidentally drops the epsilon at exactly one site.

## Drift Observations
- `compute/mod.rs:9` — `threshold_types` is declared `mod threshold_types` (private) while all other compute submodules are `pub mod`. Types are accessible via `thresholds.rs`'s `pub use super::threshold_types::*`, so this is not a bug, but the asymmetry may surprise a future contributor. A brief comment on the `mod` line would forestall the question.
