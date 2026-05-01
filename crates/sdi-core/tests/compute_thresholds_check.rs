use std::collections::BTreeMap;

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

fn cfg_with_drift_override(drift_rate: f64, expires: &str, today: NaiveDate) -> ThresholdsInput {
    let mut cfg = ThresholdsInput::default();
    cfg.today = today;
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
    assert_eq!(r.breaches[0].category, None, "aggregate breach has no category");
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
    assert!(r.breached, "expired override must not prevent aggregate breach");
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
    s.pattern_entropy_per_category_delta = Some(BTreeMap::from([
        ("error_handling".to_string(), 3.0),
    ]));
    let r = compute_thresholds_check(&s, &cfg);
    assert!(!r.breached, "active override at 50.0 prevents per-category breach of 3.0");
    let cat_breaches: Vec<_> = r.breaches.iter()
        .filter(|b| b.category.as_deref() == Some("error_handling"))
        .collect();
    assert!(cat_breaches.is_empty(), "error_handling must not breach with override limit 50.0");
    assert!(r.applied_overrides["error_handling"].active);
}

#[test]
fn expired_override_falls_back_to_global_rate() {
    // Override is expired (today 2021-01-01 > expires 2020-01-01).
    // Global rate 2.0 applies → per-category entropy 3.0 breaches.
    let today = NaiveDate::from_ymd_opt(2021, 1, 1).unwrap();
    let cfg = cfg_with_override(50.0, "2020-01-01", today);
    let mut s = summary(None, None, None, None);
    s.pattern_entropy_per_category_delta = Some(BTreeMap::from([
        ("error_handling".to_string(), 3.0),
    ]));
    let r = compute_thresholds_check(&s, &cfg);
    assert!(r.breached, "expired override must not suppress per-category breach");
    let cat_breach = r.breaches.iter()
        .find(|b| b.category.as_deref() == Some("error_handling"))
        .expect("must have per-category breach for error_handling");
    assert!((cat_breach.limit - 2.0).abs() < 1e-10,
        "expired override: limit must be global rate 2.0, got {}", cat_breach.limit);
    assert!(!r.applied_overrides["error_handling"].active);
}

// ── New M14 tests ─────────────────────────────────────────────────────────────

#[test]
fn active_override_blocks_per_category_breach() {
    // Per-category entropy 3.0 would breach the global rate 2.0.
    // An active override at 50.0 for "error_handling" prevents the breach.
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let cfg = cfg_with_override(50.0, "2030-01-01", today);
    let mut s = summary(None, None, None, None);
    s.pattern_entropy_per_category_delta = Some(BTreeMap::from([
        ("error_handling".to_string(), 3.0),
    ]));
    let r = compute_thresholds_check(&s, &cfg);
    assert!(!r.breached,
        "active override must block per-category breach (3.0 < 50.0)");
}

#[test]
fn aggregate_dimension_uses_global_rate_when_only_one_category_is_overridden() {
    // Override raises error_handling limit to 50.0.
    // The aggregate pattern_entropy_delta of 3.0 must still breach (global rate 2.0).
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let cfg = cfg_with_override(50.0, "2030-01-01", today);
    let s = summary(Some(3.0), None, None, None);
    let r = compute_thresholds_check(&s, &cfg);
    assert!(r.breached, "aggregate must breach even when per-category override is active");
    let agg_breach = r.breaches.iter()
        .find(|b| b.category.is_none() && b.dimension == "pattern_entropy")
        .expect("must have aggregate breach with no category");
    assert!((agg_breach.limit - 2.0).abs() < 1e-10,
        "aggregate limit must be global 2.0, not override 50.0");
}

#[test]
fn applied_overrides_reports_active_and_expired_separately() {
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let mut cfg = ThresholdsInput { today, ..ThresholdsInput::default() };
    cfg.overrides.insert("active_cat".to_string(), ThresholdOverrideInput {
        pattern_entropy_rate: Some(10.0),
        convention_drift_rate: None,
        coupling_delta_rate: None,
        boundary_violation_rate: None,
        expires: "2030-01-01".to_string(),
    });
    cfg.overrides.insert("expired_cat".to_string(), ThresholdOverrideInput {
        pattern_entropy_rate: Some(10.0),
        convention_drift_rate: None,
        coupling_delta_rate: None,
        boundary_violation_rate: None,
        expires: "2020-01-01".to_string(),
    });
    let r = compute_thresholds_check(&null_summary(), &cfg);
    assert!(r.applied_overrides["active_cat"].active,
        "active_cat override must be active");
    assert!(!r.applied_overrides["expired_cat"].active,
        "expired_cat override must be inactive");
    assert!(r.applied_overrides["expired_cat"].expired_reason.is_some(),
        "expired override must have an expired_reason");
}

#[test]
fn expiry_boundary_today_equals_expires_is_still_active() {
    // today == expires → override is still active (only today > expires → expired).
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let cfg = cfg_with_override(50.0, "2026-05-01", today);
    let mut s = summary(None, None, None, None);
    s.pattern_entropy_per_category_delta = Some(BTreeMap::from([
        ("error_handling".to_string(), 3.0),
    ]));
    let r = compute_thresholds_check(&s, &cfg);
    assert!(!r.breached,
        "today == expires: override still active, 3.0 < 50.0 must not breach");
    assert!(r.applied_overrides["error_handling"].active,
        "override must be active when today == expires");
}

#[test]
fn category_present_in_curr_only_surfaces_positive_delta() {
    // A category in curr but not in prev appears as positive delta in per-category maps.
    // This test verifies compute_thresholds_check correctly evaluates it.
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let cfg = ThresholdsInput { today, ..ThresholdsInput::default() };
    let mut s = summary(None, None, None, None);
    // New category "new_cat" appears for the first time → positive delta.
    s.pattern_entropy_per_category_delta = Some(BTreeMap::from([
        ("new_cat".to_string(), 3.0), // > global rate 2.0 → breach
    ]));
    let r = compute_thresholds_check(&s, &cfg);
    assert!(r.breached, "newly-appearing category with delta > global rate must breach");
    assert_eq!(r.breaches[0].category.as_deref(), Some("new_cat"));
}

#[test]
fn category_present_in_prev_only_surfaces_negative_delta() {
    // A category in prev but absent from curr has a negative per-category delta.
    // Negative deltas must never breach any threshold (rate limits are always positive).
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let cfg = ThresholdsInput { today, ..ThresholdsInput::default() };
    let mut s = summary(None, None, None, None);
    // Category disappeared from curr → negative delta (computed by compute_delta).
    s.pattern_entropy_per_category_delta = Some(BTreeMap::from([
        ("gone_cat".to_string(), -3.0), // negative → cannot exceed any positive rate
    ]));
    let r = compute_thresholds_check(&s, &cfg);
    assert!(!r.breached, "negative per-category delta must never breach a threshold");
}

// ── convention_drift_rate override wiring (thresholds.rs:220-235) ─────────────

#[test]
fn convention_drift_active_override_raises_per_category_limit() {
    // Active convention_drift_rate override at 50.0 for "error_handling".
    // Per-category drift of 4.0 does NOT breach (50.0 >> 4.0 > global 3.0).
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let cfg = cfg_with_drift_override(50.0, "2030-01-01", today);
    let mut s = summary(None, None, None, None);
    s.convention_drift_per_category_delta = Some(BTreeMap::from([
        ("error_handling".to_string(), 4.0),
    ]));
    let r = compute_thresholds_check(&s, &cfg);
    assert!(
        !r.breached,
        "active convention_drift_rate override at 50.0 must prevent per-category breach of 4.0"
    );
    let cat_breaches: Vec<_> = r
        .breaches
        .iter()
        .filter(|b| b.category.as_deref() == Some("error_handling"))
        .collect();
    assert!(
        cat_breaches.is_empty(),
        "error_handling must not breach with active convention_drift override limit 50.0"
    );
    assert!(
        r.applied_overrides["error_handling"].active,
        "applied_overrides must report override as active"
    );
}

#[test]
fn convention_drift_expired_override_falls_back_to_global_rate() {
    // Override is expired (today 2026-05-01 > expires 2020-01-01).
    // Global convention_drift_rate 3.0 applies → per-category drift 4.0 breaches.
    let today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let cfg = cfg_with_drift_override(50.0, "2020-01-01", today);
    let mut s = summary(None, None, None, None);
    s.convention_drift_per_category_delta = Some(BTreeMap::from([
        ("error_handling".to_string(), 4.0),
    ]));
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
    assert_eq!(
        cat_breach.dimension, "convention_drift",
        "breach dimension must be convention_drift"
    );
    assert!(
        (cat_breach.limit - 3.0).abs() < 1e-10,
        "expired override: limit must be global convention_drift_rate 3.0, got {}",
        cat_breach.limit
    );
    assert!(
        !r.applied_overrides["error_handling"].active,
        "applied_overrides must report override as inactive"
    );
}
