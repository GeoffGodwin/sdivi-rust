# Reviewer Report — M14: Per-Category Threshold Override Wiring

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `json.rs:44-46` — `exceeded` collects only `b.dimension.as_str()`, so multiple per-category breaches on the same dimension (e.g. three categories all exceeding `pattern_entropy`) produce duplicate entries in the `exceeded` array. Downstream CI consumers that de-duplicate this list will lose category granularity. The detailed `breaches` array has the full info, but the contract doc for the JSON shape should clarify this is a dimension-level summary, not a (dimension, category) list.
- `crates/sdi-core/src/input/types.rs:264-265` — `ThresholdsInput::default().today` is the far-future sentinel `9999-12-31`. The docstring calls this out, but any unit test that calls `compute_thresholds_check(&s, &ThresholdsInput::default())` with a non-null override will see all overrides as active. The existing tests that care about expiry all set `today` explicitly, so correctness is fine. Still worth noting for future test authors.
- `crates/sdi-snapshot/tests/null_vs_zero.rs` — `zero_delta_not_null_in_json` checks `coupling_delta` but doesn't verify the two new per-category delta fields serialize as non-null when populated. Both fields are `Some({})` for identical snapshots (empty map, not zero), which is correct but untested.

## Coverage Gaps
- No test exercises `convention_drift_per_category_delta` with an active or expired `convention_drift_rate` override. All override tests use `pattern_entropy_rate`. The code path at `thresholds.rs:220-235` is structurally symmetric with the entropy path and is almost certainly correct, but it has no per-category override coverage.
- `prop_per_category_delta_pure` in `prop_thresholds.rs` only exercises `pattern_entropy_per_category_delta`; a parallel property for `convention_drift_per_category_delta` with overrides would complete coverage.

## Drift Observations
- `pipeline.rs:221-257` (`compute_pattern_metrics_from_catalog`) duplicates the entropy + convention-drift logic from `sdi-core/src/compute/patterns.rs:35-82`. A parity test was added, which mitigates the risk, but the two implementations can drift silently if either is updated. Consider exposing `compute_pattern_metrics` from sdi-core with a catalog-consuming adapter, or a shared helper — this is not urgent given the parity test.
- `bindings/sdi-wasm/src/lib.rs:58-62` — The TypeScript custom section references `PatternMetricsResult` in the `Snapshot` interface, but tsify generates this type as `WasmPatternMetricsResult`. This name mismatch predates M14 and is not worsened by it, but TypeScript consumers may need a manual cast. A follow-up to align the custom-section type name with the tsify-generated name would eliminate the silent type gap.
- `crates/sdi-pipeline/src/time.rs:22-38` — The hand-rolled Julian Day Number → Gregorian calendar conversion is a non-trivial algorithm with no test coverage for edge cases (leap years, year-end, etc.). The existing doc test only checks `ends_with('Z')` and `len() == 20`. A round-trip property test or at least spot-check against known UTC timestamps would give confidence in the algorithm. This predates M14 (extracted, not introduced) so not a new concern.
