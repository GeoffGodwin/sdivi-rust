//! M20: THRESHOLD_EPSILON cross-arch stability tests.
//!
//! Verifies that deltas within epsilon of the limit do not breach, and that
//! deltas above `limit + THRESHOLD_EPSILON` still trip the gate.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use sdivi_core::compute::thresholds::{compute_thresholds_check, THRESHOLD_EPSILON};
use sdivi_core::input::ThresholdsInput;
use sdivi_snapshot::delta::DivergenceSummary;

fn summary(
    entropy: Option<f64>,
    drift: Option<f64>,
    coupling: Option<f64>,
    violations: Option<i64>,
) -> DivergenceSummary {
    DivergenceSummary {
        pattern_entropy_delta: entropy,
        convention_drift_delta: drift,
        coupling_delta: coupling,
        community_count_delta: None,
        boundary_violation_delta: violations,
        pattern_entropy_per_category_delta: None,
        convention_drift_per_category_delta: None,
    }
}

/// Regression gate: the constant must not be bumped carelessly.
#[test]
fn threshold_epsilon_value() {
    assert!(
        (THRESHOLD_EPSILON - 1e-9).abs() < f64::EPSILON,
        "THRESHOLD_EPSILON must be exactly 1e-9"
    );
}

/// delta = limit + 5e-10 (below epsilon): must NOT breach.
#[test]
fn epsilon_below_threshold_does_not_breach_pattern_entropy() {
    let cfg = ThresholdsInput {
        pattern_entropy_rate: 2.0,
        ..ThresholdsInput::default()
    };
    let s = summary(Some(2.0 + 5e-10), None, None, None);
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        !r.breached,
        "delta = limit + 5e-10 must not breach (within epsilon)"
    );
}

/// delta = limit + 2e-9 (above epsilon): must breach.
#[test]
fn epsilon_above_threshold_breaches_pattern_entropy() {
    let cfg = ThresholdsInput {
        pattern_entropy_rate: 2.0,
        ..ThresholdsInput::default()
    };
    let s = summary(Some(2.0 + 2e-9), None, None, None);
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        r.breached,
        "delta = limit + 2e-9 must breach (exceeds epsilon)"
    );
    assert_eq!(r.breaches[0].dimension, "pattern_entropy");
    // actual is the raw delta, not rounded
    assert!((r.breaches[0].actual - (2.0 + 2e-9)).abs() < 1e-15);
}

/// Epsilon boundary tests for convention_drift.
#[test]
fn epsilon_below_threshold_does_not_breach_convention_drift() {
    let cfg = ThresholdsInput {
        convention_drift_rate: 3.0,
        ..ThresholdsInput::default()
    };
    let s = summary(None, Some(3.0 + 5e-10), None, None);
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        !r.breached,
        "convention_drift: delta = limit + 5e-10 must not breach"
    );
}

#[test]
fn epsilon_above_threshold_breaches_convention_drift() {
    let cfg = ThresholdsInput {
        convention_drift_rate: 3.0,
        ..ThresholdsInput::default()
    };
    let s = summary(None, Some(3.0 + 2e-9), None, None);
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        r.breached,
        "convention_drift: delta = limit + 2e-9 must breach"
    );
    assert_eq!(r.breaches[0].dimension, "convention_drift");
}

/// Epsilon boundary tests for coupling_delta.
#[test]
fn epsilon_below_threshold_does_not_breach_coupling_delta() {
    let cfg = ThresholdsInput {
        coupling_delta_rate: 0.15,
        ..ThresholdsInput::default()
    };
    let s = summary(None, None, Some(0.15 + 5e-10), None);
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        !r.breached,
        "coupling_delta: delta = limit + 5e-10 must not breach"
    );
}

#[test]
fn epsilon_above_threshold_breaches_coupling_delta() {
    let cfg = ThresholdsInput {
        coupling_delta_rate: 0.15,
        ..ThresholdsInput::default()
    };
    let s = summary(None, None, Some(0.15 + 2e-9), None);
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        r.breached,
        "coupling_delta: delta = limit + 2e-9 must breach"
    );
    assert_eq!(r.breaches[0].dimension, "coupling_delta");
}

/// Per-category override path: epsilon boundaries on pattern_entropy.
#[test]
fn epsilon_below_threshold_does_not_breach_per_category() {
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let cfg = ThresholdsInput {
        today,
        ..ThresholdsInput::default()
    };
    let mut s = summary(None, None, None, None);
    s.pattern_entropy_per_category_delta =
        Some(BTreeMap::from([("errors".to_string(), 2.0 + 5e-10)]));
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        !r.breached,
        "per-category: delta = limit + 5e-10 must not breach"
    );
}

#[test]
fn epsilon_above_threshold_breaches_per_category() {
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let cfg = ThresholdsInput {
        today,
        ..ThresholdsInput::default()
    };
    let mut s = summary(None, None, None, None);
    s.pattern_entropy_per_category_delta =
        Some(BTreeMap::from([("errors".to_string(), 2.0 + 2e-9)]));
    let r = compute_thresholds_check(&s, &cfg);
    assert!(r.breached, "per-category: delta = limit + 2e-9 must breach");
    assert_eq!(r.breaches[0].category.as_deref(), Some("errors"));
}
