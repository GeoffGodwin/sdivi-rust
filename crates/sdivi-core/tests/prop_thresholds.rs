//! Property test: `compute_thresholds_check` is a pure function.
//!
//! Same `DivergenceSummary` + same `ThresholdsInput` always produces the same
//! `ThresholdCheckResult` — no side effects, no global state, no clock.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use proptest::prelude::*;
use sdivi_core::compute::thresholds::{compute_thresholds_check, THRESHOLD_EPSILON};
use sdivi_core::input::{ThresholdOverrideInput, ThresholdsInput};
use sdivi_snapshot::delta::DivergenceSummary;

fn arb_summary() -> impl Strategy<Value = DivergenceSummary> {
    (
        proptest::option::of(-5.0f64..10.0),
        proptest::option::of(-5.0f64..10.0),
        proptest::option::of(-1.0f64..1.0),
        proptest::option::of(-10i64..10),
        proptest::option::of(-5i64..20),
    )
        .prop_map(|(ped, cdd, cd, ccd, bvd)| DivergenceSummary {
            pattern_entropy_delta: ped,
            convention_drift_delta: cdd,
            coupling_delta: cd,
            community_count_delta: ccd,
            boundary_violation_delta: bvd,
            pattern_entropy_per_category_delta: None,
            convention_drift_per_category_delta: None,
        })
}

fn arb_thresholds() -> impl Strategy<Value = ThresholdsInput> {
    (0.1f64..10.0, 0.1f64..10.0, 0.01f64..1.0, 0.1f64..10.0).prop_map(|(pe, cd, coup, bv)| {
        ThresholdsInput {
            pattern_entropy_rate: pe,
            convention_drift_rate: cd,
            coupling_delta_rate: coup,
            boundary_violation_rate: bv,
            ..ThresholdsInput::default()
        }
    })
}

/// An override with either a future or past date.
fn arb_override_entry() -> impl Strategy<Value = (String, ThresholdOverrideInput)> {
    ("[a-z]{4,8}", 0.1f64..20.0, prop::bool::ANY).prop_map(|(cat, rate, active)| {
        let expires = if active { "2099-12-31" } else { "2000-01-01" };
        (
            cat,
            ThresholdOverrideInput {
                pattern_entropy_rate: Some(rate),
                convention_drift_rate: None,
                coupling_delta_rate: None,
                boundary_violation_rate: None,
                expires: expires.to_string(),
            },
        )
    })
}

fn arb_thresholds_with_overrides() -> impl Strategy<Value = ThresholdsInput> {
    (
        arb_thresholds(),
        prop::collection::vec(arb_override_entry(), 0..4),
    )
        .prop_map(|(mut cfg, entries)| {
            cfg.overrides = entries.into_iter().collect();
            // Use a fixed "today" so the test is deterministic across runs.
            cfg.today = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
            cfg
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// `compute_thresholds_check` is referentially transparent.
    #[test]
    fn prop_test_compute_thresholds_check_pure(
        summary in arb_summary(),
        thresholds in arb_thresholds(),
    ) {
        let r1 = compute_thresholds_check(&summary, &thresholds);
        let r2 = compute_thresholds_check(&summary, &thresholds);
        prop_assert_eq!(
            serde_json::to_string(&r1).unwrap(),
            serde_json::to_string(&r2).unwrap(),
            "compute_thresholds_check must be referentially transparent",
        );
    }

    /// If a delta is `None`, the corresponding dimension is never breached.
    #[test]
    fn prop_none_delta_never_breaches(thresholds in arb_thresholds()) {
        let summary = DivergenceSummary {
            pattern_entropy_delta: None,
            convention_drift_delta: None,
            coupling_delta: None,
            community_count_delta: None,
            boundary_violation_delta: None,
            pattern_entropy_per_category_delta: None,
            convention_drift_per_category_delta: None,
        };
        let r = compute_thresholds_check(&summary, &thresholds);
        prop_assert!(!r.breached, "null summary must never breach");
    }

    /// With overrides (mixed active/expired), `compute_thresholds_check` is still pure.
    #[test]
    fn prop_thresholds_with_overrides_pure(
        summary in arb_summary(),
        thresholds in arb_thresholds_with_overrides(),
    ) {
        let r1 = compute_thresholds_check(&summary, &thresholds);
        let r2 = compute_thresholds_check(&summary, &thresholds);
        prop_assert_eq!(
            serde_json::to_string(&r1).unwrap(),
            serde_json::to_string(&r2).unwrap()
        );
    }

    /// Per-category delta maps produce at most one breach per category per dimension.
    #[test]
    fn prop_per_category_delta_pure(
        cat in "[a-z]{4,8}",
        delta in -5.0f64..10.0,
        thresholds in arb_thresholds_with_overrides(),
    ) {
        let summary = DivergenceSummary {
            pattern_entropy_delta: None,
            convention_drift_delta: None,
            coupling_delta: None,
            community_count_delta: None,
            boundary_violation_delta: None,
            pattern_entropy_per_category_delta: Some(BTreeMap::from([(cat.clone(), delta)])),
            convention_drift_per_category_delta: None,
        };
        let r1 = compute_thresholds_check(&summary, &thresholds);
        let r2 = compute_thresholds_check(&summary, &thresholds);
        prop_assert_eq!(
            serde_json::to_string(&r1).unwrap(),
            serde_json::to_string(&r2).unwrap()
        );
        // At most one per-category breach for the one category we supplied.
        let cat_breaches: Vec<_> = r1.breaches.iter()
            .filter(|b| b.category.as_deref() == Some(&cat))
            .collect();
        prop_assert!(cat_breaches.len() <= 1,
            "at most one breach per category per dimension");
    }

    /// Parallel to `prop_per_category_delta_pure` but for `convention_drift_per_category_delta`.
    ///
    /// Verifies that the code path at `thresholds.rs:220-235` is also referentially transparent,
    /// and that at most one breach is produced per category for the convention_drift dimension.
    #[test]
    fn prop_convention_drift_per_category_delta_pure(
        cat in "[a-z]{4,8}",
        delta in -5.0f64..10.0,
        thresholds in arb_thresholds_with_overrides(),
    ) {
        let summary = DivergenceSummary {
            pattern_entropy_delta: None,
            convention_drift_delta: None,
            coupling_delta: None,
            community_count_delta: None,
            boundary_violation_delta: None,
            pattern_entropy_per_category_delta: None,
            convention_drift_per_category_delta: Some(BTreeMap::from([(cat.clone(), delta)])),
        };
        let r1 = compute_thresholds_check(&summary, &thresholds);
        let r2 = compute_thresholds_check(&summary, &thresholds);
        prop_assert_eq!(
            serde_json::to_string(&r1).unwrap(),
            serde_json::to_string(&r2).unwrap(),
            "convention_drift per-category check must be referentially transparent"
        );
        // At most one per-category breach for the one category we supplied.
        let cat_breaches: Vec<_> = r1.breaches.iter()
            .filter(|b| b.dimension == "convention_drift"
                && b.category.as_deref() == Some(&cat))
            .collect();
        prop_assert!(
            cat_breaches.len() <= 1,
            "at most one convention_drift breach per category"
        );
        // Negative deltas must never produce a breach (all rates are positive).
        if delta <= 0.0 {
            prop_assert!(
                cat_breaches.is_empty(),
                "negative convention_drift delta must never breach"
            );
        }
    }

    /// The pattern_entropy aggregate breach decision equals `delta > limit + THRESHOLD_EPSILON`.
    ///
    /// Trivially true given the implementation but catches accidental refactors.
    #[test]
    fn prop_breach_equals_delta_gt_limit_plus_epsilon(
        limit in 0.1f64..10.0,
        delta in -5.0f64..15.0,
    ) {
        let mut cfg = ThresholdsInput::default();
        cfg.pattern_entropy_rate = limit;
        let summary = DivergenceSummary {
            pattern_entropy_delta: Some(delta),
            convention_drift_delta: None,
            coupling_delta: None,
            community_count_delta: None,
            boundary_violation_delta: None,
            pattern_entropy_per_category_delta: None,
            convention_drift_per_category_delta: None,
        };
        let r = compute_thresholds_check(&summary, &cfg);
        let expected = delta > limit + THRESHOLD_EPSILON;
        prop_assert_eq!(
            r.breached,
            expected,
            "breached({}, {}) must equal delta > limit + THRESHOLD_EPSILON ({})",
            delta, limit, expected
        );
    }

    /// The convention_drift aggregate breach decision equals `delta > limit + THRESHOLD_EPSILON`.
    ///
    /// Guards against a per-dimension refactor that accidentally drops epsilon at the
    /// convention_drift comparison site.
    #[test]
    fn prop_breach_equals_delta_gt_limit_plus_epsilon_convention_drift(
        limit in 0.1f64..10.0,
        delta in -5.0f64..15.0,
    ) {
        let mut cfg = ThresholdsInput::default();
        cfg.convention_drift_rate = limit;
        let summary = DivergenceSummary {
            pattern_entropy_delta: None,
            convention_drift_delta: Some(delta),
            coupling_delta: None,
            community_count_delta: None,
            boundary_violation_delta: None,
            pattern_entropy_per_category_delta: None,
            convention_drift_per_category_delta: None,
        };
        let r = compute_thresholds_check(&summary, &cfg);
        let expected = delta > limit + THRESHOLD_EPSILON;
        prop_assert_eq!(
            r.breached,
            expected,
            "convention_drift: breached({}, {}) must equal delta > limit + THRESHOLD_EPSILON ({})",
            delta, limit, expected
        );
    }

    /// The coupling_delta aggregate breach decision equals `delta > limit + THRESHOLD_EPSILON`.
    ///
    /// Guards against a per-dimension refactor that accidentally drops epsilon at the
    /// coupling_delta comparison site.
    #[test]
    fn prop_breach_equals_delta_gt_limit_plus_epsilon_coupling_delta(
        limit in 0.01f64..1.0,
        delta in -1.0f64..2.0,
    ) {
        let mut cfg = ThresholdsInput::default();
        cfg.coupling_delta_rate = limit;
        let summary = DivergenceSummary {
            pattern_entropy_delta: None,
            convention_drift_delta: None,
            coupling_delta: Some(delta),
            community_count_delta: None,
            boundary_violation_delta: None,
            pattern_entropy_per_category_delta: None,
            convention_drift_per_category_delta: None,
        };
        let r = compute_thresholds_check(&summary, &cfg);
        let expected = delta > limit + THRESHOLD_EPSILON;
        prop_assert_eq!(
            r.breached,
            expected,
            "coupling_delta: breached({}, {}) must equal delta > limit + THRESHOLD_EPSILON ({})",
            delta, limit, expected
        );
    }

    /// The boundary_violation_delta aggregate breach decision equals
    /// `(delta as f64) > limit + THRESHOLD_EPSILON`.
    ///
    /// `boundary_violation_delta` is `i64`; epsilon has no functional effect for
    /// integer-cast deltas, but the same comparison form is applied for consistency.
    /// This test guards against a future refactor that drops epsilon at this site.
    #[test]
    fn prop_breach_equals_delta_gt_limit_plus_epsilon_boundary_violation(
        limit in 0.1f64..10.0,
        delta in -10i64..20,
    ) {
        let mut cfg = ThresholdsInput::default();
        cfg.boundary_violation_rate = limit;
        let summary = DivergenceSummary {
            pattern_entropy_delta: None,
            convention_drift_delta: None,
            coupling_delta: None,
            community_count_delta: None,
            boundary_violation_delta: Some(delta),
            pattern_entropy_per_category_delta: None,
            convention_drift_per_category_delta: None,
        };
        let r = compute_thresholds_check(&summary, &cfg);
        let delta_f = delta as f64;
        let expected = delta_f > limit + THRESHOLD_EPSILON;
        prop_assert_eq!(
            r.breached,
            expected,
            "boundary_violation: breached({}, {}) must equal delta_f > limit + THRESHOLD_EPSILON ({})",
            delta_f, limit, expected
        );
    }
}
