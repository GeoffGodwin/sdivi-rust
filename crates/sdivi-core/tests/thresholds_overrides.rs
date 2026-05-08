//! Per-category override tests for `compute_thresholds_check`.
//!
//! Covers M14 override wiring: active/expired overrides for `pattern_entropy_rate`
//! and `convention_drift_rate` per-category dimensions.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use sdivi_core::compute::thresholds::compute_thresholds_check;
use sdivi_core::input::{ThresholdOverrideInput, ThresholdsInput};
use sdivi_snapshot::delta::{null_summary, DivergenceSummary};

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

fn cfg_with_override(entropy_rate: f64, expires: &str, today: NaiveDate) -> ThresholdsInput {
    let mut cfg = ThresholdsInput {
        today,
        ..ThresholdsInput::default()
    };
    cfg.overrides.insert(
        "error_handling".to_string(),
        ThresholdOverrideInput {
            pattern_entropy_rate: Some(entropy_rate),
            convention_drift_rate: None,
            coupling_delta_rate: None,
            boundary_violation_rate: None,
            expires: expires.to_string(),
        },
    );
    cfg
}

fn cfg_with_drift_override(drift_rate: f64, expires: &str, today: NaiveDate) -> ThresholdsInput {
    let mut cfg = ThresholdsInput {
        today,
        ..ThresholdsInput::default()
    };
    cfg.overrides.insert(
        "error_handling".to_string(),
        ThresholdOverrideInput {
            pattern_entropy_rate: None,
            convention_drift_rate: Some(drift_rate),
            coupling_delta_rate: None,
            boundary_violation_rate: None,
            expires: expires.to_string(),
        },
    );
    cfg
}

#[test]
fn active_override_blocks_per_category_breach() {
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let cfg = cfg_with_override(50.0, "2030-01-01", today);
    let mut s = summary(None, None, None, None);
    s.pattern_entropy_per_category_delta =
        Some(BTreeMap::from([("error_handling".to_string(), 3.0)]));
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        !r.breached,
        "active override must block per-category breach (3.0 < 50.0)"
    );
}

#[test]
fn aggregate_dimension_uses_global_rate_when_only_one_category_is_overridden() {
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let cfg = cfg_with_override(50.0, "2030-01-01", today);
    let s = summary(Some(3.0), None, None, None);
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        r.breached,
        "aggregate must breach even when per-category override is active"
    );
    let agg_breach = r
        .breaches
        .iter()
        .find(|b| b.category.is_none() && b.dimension == "pattern_entropy")
        .expect("must have aggregate breach with no category");
    assert!(
        (agg_breach.limit - 2.0).abs() < 1e-10,
        "aggregate limit must be global 2.0, not override 50.0"
    );
}

#[test]
fn applied_overrides_reports_active_and_expired_separately() {
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let mut cfg = ThresholdsInput {
        today,
        ..ThresholdsInput::default()
    };
    cfg.overrides.insert(
        "active_cat".to_string(),
        ThresholdOverrideInput {
            pattern_entropy_rate: Some(10.0),
            convention_drift_rate: None,
            coupling_delta_rate: None,
            boundary_violation_rate: None,
            expires: "2030-01-01".to_string(),
        },
    );
    cfg.overrides.insert(
        "expired_cat".to_string(),
        ThresholdOverrideInput {
            pattern_entropy_rate: Some(10.0),
            convention_drift_rate: None,
            coupling_delta_rate: None,
            boundary_violation_rate: None,
            expires: "2020-01-01".to_string(),
        },
    );
    let r = compute_thresholds_check(&null_summary(), &cfg);
    assert!(r.applied_overrides["active_cat"].active);
    assert!(!r.applied_overrides["expired_cat"].active);
    assert!(r.applied_overrides["expired_cat"].expired_reason.is_some());
}

#[test]
fn expiry_boundary_today_equals_expires_is_still_active() {
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let cfg = cfg_with_override(50.0, "2026-05-01", today);
    let mut s = summary(None, None, None, None);
    s.pattern_entropy_per_category_delta =
        Some(BTreeMap::from([("error_handling".to_string(), 3.0)]));
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        !r.breached,
        "today == expires: override still active, 3.0 < 50.0 must not breach"
    );
    assert!(r.applied_overrides["error_handling"].active);
}

#[test]
fn category_present_in_curr_only_surfaces_positive_delta() {
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let cfg = ThresholdsInput {
        today,
        ..ThresholdsInput::default()
    };
    let mut s = summary(None, None, None, None);
    s.pattern_entropy_per_category_delta = Some(BTreeMap::from([("new_cat".to_string(), 3.0)]));
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        r.breached,
        "new category with delta > global rate must breach"
    );
    assert_eq!(r.breaches[0].category.as_deref(), Some("new_cat"));
}

#[test]
fn category_present_in_prev_only_surfaces_negative_delta() {
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let cfg = ThresholdsInput {
        today,
        ..ThresholdsInput::default()
    };
    let mut s = summary(None, None, None, None);
    s.pattern_entropy_per_category_delta = Some(BTreeMap::from([("gone_cat".to_string(), -3.0)]));
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        !r.breached,
        "negative per-category delta must never breach a threshold"
    );
}

#[test]
fn convention_drift_active_override_raises_per_category_limit() {
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let cfg = cfg_with_drift_override(50.0, "2030-01-01", today);
    let mut s = summary(None, None, None, None);
    s.convention_drift_per_category_delta =
        Some(BTreeMap::from([("error_handling".to_string(), 4.0)]));
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        !r.breached,
        "active convention_drift_rate override at 50.0 must prevent breach of 4.0"
    );
    let cat_breaches: Vec<_> = r
        .breaches
        .iter()
        .filter(|b| b.category.as_deref() == Some("error_handling"))
        .collect();
    assert!(cat_breaches.is_empty());
    assert!(r.applied_overrides["error_handling"].active);
}

#[test]
fn convention_drift_expired_override_falls_back_to_global_rate() {
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let cfg = cfg_with_drift_override(50.0, "2020-01-01", today);
    let mut s = summary(None, None, None, None);
    s.convention_drift_per_category_delta =
        Some(BTreeMap::from([("error_handling".to_string(), 4.0)]));
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        r.breached,
        "expired convention_drift_rate override must not suppress per-category breach"
    );
    let cat_breach = r
        .breaches
        .iter()
        .find(|b| b.category.as_deref() == Some("error_handling"))
        .expect("must have a per-category breach for error_handling");
    assert_eq!(cat_breach.dimension, "convention_drift");
    assert!(
        (cat_breach.limit - 3.0).abs() < 1e-10,
        "expired override: limit must be global 3.0, got {}",
        cat_breach.limit
    );
    assert!(!r.applied_overrides["error_handling"].active);
}
