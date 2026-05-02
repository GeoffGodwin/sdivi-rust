// Tests for ThresholdsInput default behavior and override sentinel value

use chrono::NaiveDate;
use sdivi_core::input::{ThresholdOverrideInput, ThresholdsInput};
use std::collections::BTreeMap;

/// Verify that ThresholdsInput::default() sets today to far-future sentinel (9999-12-31).
/// This conservative failure mode ensures all overrides are expired by default.
#[test]
fn default_today_is_far_future_sentinel() {
    let input = ThresholdsInput::default();
    let expected = NaiveDate::from_ymd_opt(9999, 12, 31).unwrap();
    assert_eq!(
        input.today, expected,
        "ThresholdsInput::default().today should be 9999-12-31 (far future sentinel)"
    );
}

/// Verify that default global rates are applied correctly.
#[test]
fn default_global_rates() {
    let input = ThresholdsInput::default();
    assert_eq!(
        input.pattern_entropy_rate, 2.0,
        "Default pattern_entropy_rate"
    );
    assert_eq!(
        input.convention_drift_rate, 3.0,
        "Default convention_drift_rate"
    );
    assert_eq!(
        input.coupling_delta_rate, 0.15,
        "Default coupling_delta_rate"
    );
    assert_eq!(
        input.boundary_violation_rate, 2.0,
        "Default boundary_violation_rate"
    );
}

/// Verify that the default has an empty overrides map.
#[test]
fn default_overrides_empty() {
    let input = ThresholdsInput::default();
    assert!(
        input.overrides.is_empty(),
        "Default should have no overrides"
    );
}

/// Verify that a caller can set an explicit today date on ThresholdsInput.
#[test]
fn caller_can_set_explicit_today_on_thresholds_input() {
    let today_real = NaiveDate::from_ymd_opt(2026, 4, 30).unwrap();
    let input = ThresholdsInput {
        today: today_real,
        ..ThresholdsInput::default()
    };

    assert_eq!(input.today, today_real, "Caller can set explicit today");
    assert_eq!(input.pattern_entropy_rate, 2.0, "Global rates unchanged");
}

/// Verify that ThresholdsInput can be constructed with custom overrides
/// to document the API contract.
#[test]
fn thresholds_input_with_active_override() {
    let today = NaiveDate::from_ymd_opt(2026, 4, 30).unwrap();
    let mut overrides = BTreeMap::new();
    overrides.insert(
        "migration_in_progress".to_string(),
        ThresholdOverrideInput {
            pattern_entropy_rate: Some(5.0),
            convention_drift_rate: Some(4.0),
            coupling_delta_rate: None,
            boundary_violation_rate: None,
            expires: "2026-09-30".to_string(),
        },
    );

    let input = ThresholdsInput {
        today,
        overrides,
        ..ThresholdsInput::default()
    };

    assert_eq!(input.today, today);
    assert_eq!(input.overrides.len(), 1);
    let override_entry = &input.overrides["migration_in_progress"];
    assert_eq!(override_entry.pattern_entropy_rate, Some(5.0));
    assert_eq!(override_entry.expires, "2026-09-30");
    // Expiry evaluation (whether the override applies) happens in compute_thresholds_check,
    // not in ThresholdsInput itself.
}
