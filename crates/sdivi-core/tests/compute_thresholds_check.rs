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

// ── First-snapshot path ───────────────────────────────────────────────────────

#[test]
fn null_summary_never_breaches() {
    let r = compute_thresholds_check(&null_summary(), &ThresholdsInput::default());
    assert!(!r.breached, "first-snapshot must never breach");
    assert!(r.breaches.is_empty());
}

// ── Basic aggregate breach detection ─────────────────────────────────────────

#[test]
fn entropy_breach_detected() {
    let s = summary(Some(3.0), None, None, None);
    let r = compute_thresholds_check(&s, &ThresholdsInput::default());
    assert!(r.breached);
    assert_eq!(r.breaches[0].dimension, "pattern_entropy");
    assert_eq!(
        r.breaches[0].category, None,
        "aggregate breach has no category"
    );
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

#[test]
fn custom_rate_raises_aggregate_limit() {
    let mut cfg = ThresholdsInput::default();
    cfg.pattern_entropy_rate = 10.0;
    let s = summary(Some(5.0), None, None, None);
    let r = compute_thresholds_check(&s, &cfg);
    assert!(!r.breached);
}

// ── Override expiry ───────────────────────────────────────────────────────────

#[test]
fn override_expiry_ignored_when_expired() {
    // Override sets high limit (50.0) but is expired — aggregate uses default 2.0.
    let today = NaiveDate::from_ymd_opt(2026, 4, 29).unwrap();
    let cfg = cfg_with_override(50.0, "2020-01-01", today);
    let s = summary(Some(3.0), None, None, None);
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        r.breached,
        "expired override must not prevent aggregate breach"
    );
    assert!(
        !r.applied_overrides["error_handling"].active,
        "expired override must be marked inactive in applied_overrides"
    );
}

// ── Override wiring tests ─────────────────────────────────────────────────────

#[test]
fn active_override_raises_per_category_limit() {
    // Active override at 50.0 for "error_handling": per-category entropy of 3.0
    // does NOT breach (50.0 >> 3.0). Global rate is still 2.0 for the aggregate.
    let today = NaiveDate::from_ymd_opt(2026, 4, 29).unwrap();
    let cfg = cfg_with_override(50.0, "2030-01-01", today);
    let mut s = summary(None, None, None, None);
    s.pattern_entropy_per_category_delta =
        Some(BTreeMap::from([("error_handling".to_string(), 3.0)]));
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        !r.breached,
        "active override at 50.0 prevents per-category breach of 3.0"
    );
    let cat_breaches: Vec<_> = r
        .breaches
        .iter()
        .filter(|b| b.category.as_deref() == Some("error_handling"))
        .collect();
    assert!(
        cat_breaches.is_empty(),
        "error_handling must not breach with override limit 50.0"
    );
    assert!(r.applied_overrides["error_handling"].active);
}

#[test]
fn expired_override_falls_back_to_global_rate() {
    // Override is expired (today 2021-01-01 > expires 2020-01-01).
    // Global rate 2.0 applies → per-category entropy 3.0 breaches.
    let today = NaiveDate::from_ymd_opt(2021, 1, 1).unwrap();
    let cfg = cfg_with_override(50.0, "2020-01-01", today);
    let mut s = summary(None, None, None, None);
    s.pattern_entropy_per_category_delta =
        Some(BTreeMap::from([("error_handling".to_string(), 3.0)]));
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        r.breached,
        "expired override must not suppress per-category breach"
    );
    let cat_breach = r
        .breaches
        .iter()
        .find(|b| b.category.as_deref() == Some("error_handling"))
        .expect("must have per-category breach for error_handling");
    assert!(
        (cat_breach.limit - 2.0).abs() < 1e-10,
        "expired override: limit must be global rate 2.0, got {}",
        cat_breach.limit
    );
    assert!(!r.applied_overrides["error_handling"].active);
}
