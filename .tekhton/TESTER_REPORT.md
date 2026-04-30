## Planned Tests
- [x] `crates/sdi-core/tests/thresholds_input_default_behavior.rs` — Verify ThresholdsInput default sentinel value and override filtering behavior
- [x] `crates/sdi-config/tests/threshold_overrides.rs` — Existing comprehensive test suite for threshold override expiry evaluation
- [x] `crates/sdi-core/tests/compute_thresholds_check.rs` — Existing tests verify override expiry behavior with today date
- [x] `crates/sdi-cli/tests/version.rs` — Fixed stale version expectation (0.0.9 → 0.0.10)

## Test Run Results
Passed: 494  Failed: 0

## Bugs Found
None

## Files Modified
- [x] `crates/sdi-core/tests/thresholds_input_default_behavior.rs`
- [x] `crates/sdi-cli/tests/version.rs`
- [x] `CHANGELOG.md`

## Audit Rework
- [x] Fixed INTEGRITY finding: Deleted `far_future_sentinel_makes_all_overrides_appear_expired` test in `crates/sdi-core/tests/thresholds_input_default_behavior.rs:45` — test only asserted struct construction (assertEqual(x, x)), never called compute_thresholds_check. Real behavior already covered by `override_expiry_ignored_when_expired`.
- [x] Fixed NAMING finding: Renamed `override_applied_when_not_expired` → `override_not_wired_in_m08_base_rate_applies` in `crates/sdi-core/tests/compute_thresholds_check.rs:107` — name contradicted assertion ("override applied" vs "base rate applies, override not wired"). Added TODO(M09) comment for when overrides are wired.
- [x] Fixed NAMING finding: Renamed `expired_today_date_consistent` → `base_rate_applies_regardless_of_override_state_m08` in `crates/sdi-core/tests/compute_thresholds_check.rs:120` — test name claimed to verify clock independence, but M08 doesn't read cfg.today for override filtering. Added TODO(M09) comment explaining clock-independence verification must wait until overrides are wired.
- [x] Fixed COVERAGE finding: Renamed `caller_can_supply_real_today_to_enable_overrides` → `caller_can_set_explicit_today_on_thresholds_input` in `crates/sdi-core/tests/thresholds_input_default_behavior.rs:77` — name implied "enabling overrides" but test only constructed struct with explicit today field. Corrected name to match actual behavior.

All remaining tests pass: 5 in `thresholds_input_default_behavior.rs`, 10 in `compute_thresholds_check.rs`.
