use chrono::NaiveDate;
use sdi_core::compute::thresholds::compute_thresholds_check;
use sdi_core::input::{ThresholdOverrideInput, ThresholdsInput};
use sdi_snapshot::delta::{DivergenceSummary, null_summary};

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
    }
}

// ── First-snapshot path ───────────────────────────────────────────────────────

#[test]
fn null_summary_never_breaches() {
    let r = compute_thresholds_check(&null_summary(), &ThresholdsInput::default());
    assert!(!r.breached, "first-snapshot must never breach");
    assert!(r.breaches.is_empty());
}

// ── Basic breach detection ────────────────────────────────────────────────────

#[test]
fn entropy_breach_detected() {
    let s = summary(Some(3.0), None, None, None);
    let r = compute_thresholds_check(&s, &ThresholdsInput::default());
    assert!(r.breached);
    assert_eq!(r.breaches[0].dimension, "pattern_entropy");
}

#[test]
fn entropy_at_limit_not_breached() {
    let s = summary(Some(2.0), None, None, None);
    let r = compute_thresholds_check(&s, &ThresholdsInput::default());
    assert!(!r.breached);
}

#[test]
fn coupling_breach_detected() {
    let s = summary(None, None, Some(0.5), None);
    let r = compute_thresholds_check(&s, &ThresholdsInput::default());
    assert!(r.breached);
    assert_eq!(r.breaches[0].dimension, "coupling_delta");
}

#[test]
fn boundary_violation_breach_detected() {
    let s = summary(None, None, None, Some(3));
    let r = compute_thresholds_check(&s, &ThresholdsInput::default());
    assert!(r.breached);
    assert_eq!(r.breaches[0].dimension, "boundary_violations");
}

#[test]
fn negative_delta_never_breaches() {
    let s = summary(Some(-10.0), Some(-10.0), Some(-0.5), Some(-5));
    let r = compute_thresholds_check(&s, &ThresholdsInput::default());
    assert!(!r.breached);
}

#[test]
fn multiple_breaches_all_reported() {
    let s = summary(Some(5.0), Some(5.0), Some(0.5), Some(10));
    let r = compute_thresholds_check(&s, &ThresholdsInput::default());
    assert_eq!(r.breaches.len(), 4);
}

// ── Override expiry ───────────────────────────────────────────────────────────

fn cfg_with_override(entropy_rate: f64, expires: &str, today: NaiveDate) -> ThresholdsInput {
    let mut cfg = ThresholdsInput::default();
    cfg.today = today;
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

#[test]
fn override_expiry_ignored_when_expired() {
    // Override sets high limit (50.0) but is expired — default 2.0 applies,
    // so entropy=3.0 should breach.
    let today = NaiveDate::from_ymd_opt(2026, 4, 29).unwrap();
    let cfg = cfg_with_override(50.0, "2020-01-01", today);
    let s = summary(Some(3.0), None, None, None);
    let r = compute_thresholds_check(&s, &cfg);
    assert!(r.breached, "expired override must not prevent breach");
}

#[test]
fn override_not_wired_in_m08_base_rate_applies() {
    // TODO(M09): When per-category overrides are wired, update this test to assert
    // that an active override raises the limit and prevents breach. Currently in M08,
    // the override is accepted but not read; base rate always applies.
    //
    // M08: per-category overrides do not affect the aggregate dimension check
    // (wired up in M09).  Even with an unexpired override raising the entropy
    // limit to 50.0, the base rate of 2.0 still applies, so entropy=3.0 breaches.
    let today = NaiveDate::from_ymd_opt(2026, 4, 29).unwrap();
    let cfg = cfg_with_override(50.0, "2030-01-01", today);
    let s = summary(Some(3.0), None, None, None);
    let r = compute_thresholds_check(&s, &cfg);
    assert!(r.breached, "M08: base rate 2.0 applies; override not yet wired");
    assert_eq!(r.breaches[0].dimension, "pattern_entropy");
    assert!((r.breaches[0].limit - 2.0).abs() < 1e-10, "base rate must be 2.0, not override rate");
}

#[test]
fn base_rate_applies_regardless_of_override_state_m08() {
    // TODO(M09): Once overrides are wired, verify that cfg.today drives expiry and not
    // SystemTime::now(). True clock-independence must wait until compute_thresholds_check
    // reads cfg.today for override filtering.
    //
    // In M08, overrides are not read, so with today=2019-12-31 (override not yet expired)
    // or today=2026-04-29 (override expired), the base rate 2.0 always applies and
    // entropy=3.0 always breaches. This test documents M08 behavior only.
    let past = NaiveDate::from_ymd_opt(2019, 12, 31).unwrap();
    let cfg = cfg_with_override(50.0, "2020-01-01", past);
    let s = summary(Some(3.0), None, None, None);
    let r = compute_thresholds_check(&s, &cfg);
    assert!(r.breached, "base rate 2.0 applies regardless of override state in M08");
    assert!((r.breaches[0].limit - 2.0).abs() < 1e-10, "limit must reflect base rate, not override");
}
