//! Property test: `compute_thresholds_check` is a pure function.
//!
//! Same `DivergenceSummary` + same `ThresholdsInput` always produces the same
//! `ThresholdCheckResult` — no side effects, no global state, no clock.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use proptest::prelude::*;
use sdi_core::compute::thresholds::compute_thresholds_check;
use sdi_core::input::{ThresholdOverrideInput, ThresholdsInput};
use sdi_snapshot::delta::DivergenceSummary;

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
    (0.1f64..10.0, 0.1f64..10.0, 0.01f64..1.0, 0.1f64..10.0).prop_map(
        |(pe, cd, coup, bv)| ThresholdsInput {
            pattern_entropy_rate: pe,
            convention_drift_rate: cd,
            coupling_delta_rate: coup,
            boundary_violation_rate: bv,
            ..ThresholdsInput::default()
        },
    )
}

/// An override with either a future or past date.
fn arb_override_entry() -> impl Strategy<Value = (String, ThresholdOverrideInput)> {
    (
        "[a-z]{4,8}",
        0.1f64..20.0,
        prop::bool::ANY,
    )
    .prop_map(|(cat, rate, active)| {
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
}
