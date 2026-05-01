## Test Audit Report

### Audit Summary
Tests audited: 3 files, 29 test functions (19 unit + 5 property + 5 integration)
Verdict: PASS

Implementation files cross-referenced: `crates/sdi-core/src/compute/thresholds.rs`,
`crates/sdi-snapshot/src/delta.rs`, `crates/sdi-snapshot/src/snapshot.rs`,
`crates/sdi-core/src/compute/patterns.rs`.

---

### Findings

#### COVERAGE: `prop_thresholds_with_overrides_pure` never exercises overrides and per-category deltas together
- File: `crates/sdi-core/tests/prop_thresholds.rs:116`
- Issue: `arb_summary()` always generates `None` for both `pattern_entropy_per_category_delta` and `convention_drift_per_category_delta`. Consequently `prop_thresholds_with_overrides_pure` only exercises `resolve_overrides` and the aggregate dimension checks — never the combination of "active override for category X + non-None delta for category X." The separate `prop_per_category_delta_pure` tests use `arb_thresholds_with_overrides()` but generate category strings independently; the probability of a randomly-generated category name in the delta map matching one in the override map is negligible (~0.02% per case at 500 runs), so the "override applies to exactly this category" path is almost never exercised by any property test. Unit tests cover this specific path (`active_override_raises_per_category_limit`, `active_override_blocks_per_category_breach`), but no property test verifies the combined behavior is referentially transparent.
- Severity: MEDIUM
- Action: Create an `arb_summary_with_categories(cats: Vec<String>)` helper that uses the supplied category names when building per-category delta maps, then pair it with `arb_thresholds_with_overrides()` using the same category pool so override-category collisions occur at usable frequency. Alternatively, add a dedicated property test that explicitly injects one known category into both the override map and the per-category delta map and asserts referential transparency.

---

#### NAMING: Two test names describe `compute_delta` behavior but exercise only `compute_thresholds_check`
- File: `crates/sdi-core/tests/compute_thresholds_check.rs:253` (`category_present_in_curr_only_surfaces_positive_delta`)
- File: `crates/sdi-core/tests/compute_thresholds_check.rs:269` (`category_present_in_prev_only_surfaces_negative_delta`)
- Issue: Both names use framing ("category present in curr only", "category present in prev only") that describes `compute_delta`'s union-of-keys semantics (`delta.rs:189-201`). A reader expects an end-to-end test that builds two snapshots with differing category sets and verifies the delta values produced. Instead, both tests manually construct a `DivergenceSummary` with a pre-set delta and only call `compute_thresholds_check`. The assertions are correct for what the tests actually do (positive delta > global rate breaches; negative delta never breaches), but the names mislead about coverage scope.
- Severity: LOW
- Action: Rename to `positive_per_category_delta_breaches_when_above_global_rate` and `negative_per_category_delta_never_breaches`. If end-to-end coverage of `compute_delta`'s curr-only / prev-only category semantics is also desired, add separate tests in `null_vs_zero.rs` that build two snapshots with differing category sets.

---

#### COVERAGE: `identical_snapshots_have_zero_deltas` omits `convention_drift_delta` assertion
- File: `crates/sdi-snapshot/tests/null_vs_zero.rs:50`
- Issue: The test asserts `coupling_delta`, `community_count_delta`, and `pattern_entropy_delta` are `Some(0.0)` / `Some(0)` for identical snapshots, but does not assert `convention_drift_delta`. `compute_delta` sets `convention_drift_delta = Some(curr.pattern_metrics.convention_drift - prev.pattern_metrics.convention_drift)`, which is `Some(0.0)` for identical snapshots. The omission leaves a gap in the null-vs-zero semantic contract for this field.
- Severity: LOW
- Action: Add `assert_eq!(delta.convention_drift_delta, Some(0.0), "convention_drift_delta must be Some(0.0) for identical snapshots");` after the existing assertions in `identical_snapshots_have_zero_deltas`.

---

#### COVERAGE: `prop_per_category_delta_pure` "at most one breach" assertion is trivially satisfied
- File: `crates/sdi-core/tests/prop_thresholds.rs:154`
- Issue: The assertion `cat_breaches.len() <= 1` ("at most one breach per category per dimension") is trivially true because exactly one category is inserted into `pattern_entropy_per_category_delta`. The implementation iterates each category once in a `for (cat, &delta) in per_cat` loop, so duplicate breaches are structurally impossible for any input. The assertion therefore does not distinguish a correct implementation from a buggy one that emits zero breaches. The referential-transparency check immediately above it (JSON equality across two calls) is the test's real value; the `<= 1` assertion adds no additional signal.
- Severity: LOW
- Action: Replace `cat_breaches.len() <= 1` with a condition that verifies the expected breach count based on the delta and effective limit: breach occurs if and only if `delta > effective_rate` (where `effective_rate` is the per-category override rate if active, else the global rate). Alternatively, drop the redundant assertion and add a comment explaining that uniqueness is guaranteed by construction.

---

### Passing Items

**INTEGRITY — PASS.** All assertions derive expected values from test inputs and documented implementation behavior. The `2.0` and `3.0` limit checks in `expired_override_falls_back_to_global_rate` and `convention_drift_expired_override_falls_back_to_global_rate` match the actual `ThresholdsInput::default()` global rates used in `thresholds.rs`. No hard-coded magic values, no tautological assertions, no always-passing assertions outside the trivially-true `<= 1` case noted above.

**WEAKENING — PASS.** The renamed stub tests (`override_not_wired_in_m08_base_rate_applies` → `active_override_raises_per_category_limit`; `base_rate_applies_regardless_of_override_state_m08` → `expired_override_falls_back_to_global_rate`) carry materially stronger assertions than the stubs they replaced. No existing assertion was removed or broadened.

**EXERCISE — PASS.** All three files call real implementation functions (`compute_thresholds_check`, `compute_delta`, `null_summary`, `assemble_snapshot`, `compute_pattern_metrics` via `PatternMetricsResult::default()`). No internal dependencies are mocked.

**SCOPE — PASS.** All imports and symbols (`DivergenceSummary`, `ThresholdsInput`, `ThresholdOverrideInput`, `null_summary`, `compute_delta`, `assemble_snapshot`, `PatternMetricsResult`) exist in the current implementation. No orphaned references to deleted or renamed symbols.

**ISOLATION — PASS.** All three test files build their own fixture data via helper functions. No test reads mutable project files (`.tekhton/`, `.claude/`, build artifacts, config state).

**NAMING — PASS (with LOW exceptions noted above).** All other test function names encode the scenario and expected outcome clearly.
